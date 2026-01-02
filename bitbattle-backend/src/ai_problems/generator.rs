use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::executor::CodeExecutor;
use crate::llm::LlmProvider;
use crate::problems::Difficulty;

use super::models::{AiProblem, NewAiProblem, PoolCounts, ProblemStatus};
use super::prompts::{build_generation_prompt, GeneratedProblem, SYSTEM_PROMPT};
use super::validator::{to_test_cases, ProblemValidator, ValidationResult};

/// Background service for generating AI problems
pub struct ProblemGenerator {
    pool: PgPool,
    llm: Arc<dyn LlmProvider>,
    validator: ProblemValidator,
    config: Arc<Config>,
    is_running: Arc<RwLock<bool>>,
}

impl ProblemGenerator {
    pub fn new(
        pool: PgPool,
        llm: Arc<dyn LlmProvider>,
        executor: Arc<CodeExecutor>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            pool,
            llm,
            validator: ProblemValidator::new(executor),
            config,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the background generation loop
    pub async fn start(self: Arc<Self>) {
        let already_running = {
            let mut running = self.is_running.write().await;
            if *running {
                true
            } else {
                *running = true;
                false
            }
        };

        if already_running {
            tracing::warn!("Problem generator already running");
            return;
        }

        tracing::info!("Starting AI problem generator");

        let interval = Duration::from_secs(self.config.ai_generation_interval_secs);

        loop {
            // Check if we should stop
            if !*self.is_running.read().await {
                break;
            }

            // Check pool levels and generate if needed
            if let Err(e) = self.check_and_generate().await {
                tracing::error!("Error in problem generation loop: {}", e);
            }

            // Also validate any pending problems
            if let Err(e) = self.validate_pending().await {
                tracing::error!("Error validating pending problems: {}", e);
            }

            tokio::time::sleep(interval).await;
        }

        tracing::info!("Problem generator stopped");
    }

    /// Stop the background generation
    pub async fn stop(&self) {
        *self.is_running.write().await = false;
    }

    /// Check pool levels and generate problems if needed
    async fn check_and_generate(&self) -> Result<(), String> {
        let counts = AiProblem::get_pool_counts(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        tracing::debug!(
            "Current pool counts - Easy: {}, Medium: {}, Hard: {}",
            counts.easy,
            counts.medium,
            counts.hard
        );

        // Check each difficulty level
        if (counts.easy as u32) < self.config.ai_min_pool_easy {
            tracing::info!(
                "Easy pool low ({}/{}), generating problem",
                counts.easy,
                self.config.ai_min_pool_easy
            );
            self.generate_problem(Difficulty::Easy).await?;
        }

        if (counts.medium as u32) < self.config.ai_min_pool_medium {
            tracing::info!(
                "Medium pool low ({}/{}), generating problem",
                counts.medium,
                self.config.ai_min_pool_medium
            );
            self.generate_problem(Difficulty::Medium).await?;
        }

        if (counts.hard as u32) < self.config.ai_min_pool_hard {
            tracing::info!(
                "Hard pool low ({}/{}), generating problem",
                counts.hard,
                self.config.ai_min_pool_hard
            );
            self.generate_problem(Difficulty::Hard).await?;
        }

        Ok(())
    }

    /// Generate a single problem of the given difficulty
    async fn generate_problem(&self, difficulty: Difficulty) -> Result<(), String> {
        let user_prompt = build_generation_prompt(difficulty.clone());

        tracing::info!(
            "Generating {:?} problem using {} ({})",
            difficulty,
            self.llm.name(),
            self.llm.model()
        );

        // Call LLM
        let response = self
            .llm
            .complete(SYSTEM_PROMPT, &user_prompt)
            .await
            .map_err(|e| format!("LLM error: {}", e))?;

        // Log token usage
        if let Some(usage) = &response.usage {
            tracing::info!(
                "LLM tokens used - prompt: {}, completion: {}, total: {}",
                usage.prompt_tokens,
                usage.completion_tokens,
                usage.total_tokens
            );
        }

        // Parse response
        let generated = GeneratedProblem::from_llm_response(&response.content)
            .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

        tracing::info!("Generated problem: {}", generated.title);

        // Generate unique problem ID
        let problem_id = format!(
            "ai-{}-{}",
            slug::slugify(&generated.title),
            chrono::Utc::now().timestamp_millis() % 10000
        );

        // Convert to NewAiProblem
        let new_problem = NewAiProblem {
            problem_id,
            title: generated.title.clone(),
            description: generated.description.clone(),
            difficulty,
            examples: to_test_cases(&generated.examples),
            test_cases: to_test_cases(&generated.test_cases),
            starter_code: generated.starter_code.clone(),
            time_limit_minutes: generated.time_limit_minutes,
            tags: generated.tags.clone(),
            provider: self.llm.name().to_string(),
            model: self.llm.model().to_string(),
        };

        // Insert into database
        let ai_problem = AiProblem::insert(&self.pool, new_problem)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        tracing::info!("Saved problem {} with status pending_validation", ai_problem.problem_id);

        // Immediately try to validate
        match self.validator.validate(&generated).await {
            ValidationResult::Valid => {
                AiProblem::update_status(&self.pool, ai_problem.id, ProblemStatus::Validated, None)
                    .await
                    .map_err(|e| format!("Failed to update status: {}", e))?;
                tracing::info!("Problem {} validated successfully", ai_problem.problem_id);
            }
            ValidationResult::Invalid(error) => {
                AiProblem::update_status(
                    &self.pool,
                    ai_problem.id,
                    ProblemStatus::PendingValidation,
                    Some(&error),
                )
                .await
                .map_err(|e| format!("Failed to update status: {}", e))?;
                tracing::warn!("Problem {} failed validation: {}", ai_problem.problem_id, error);
            }
        }

        Ok(())
    }

    /// Validate pending problems
    async fn validate_pending(&self) -> Result<(), String> {
        // Get a pending problem
        let Some(problem) = AiProblem::get_pending_for_validation(&self.pool)
            .await
            .map_err(|e| e.to_string())?
        else {
            return Ok(());
        };

        tracing::info!("Validating pending problem: {}", problem.problem_id);

        // Reconstruct GeneratedProblem for validation
        // This is a bit awkward but necessary since we need the reference solution
        // For now, we'll just skip re-validation of existing problems
        // In production, you'd store the reference solution or re-generate it

        // For problems without stored reference solution, we reject after max attempts
        if problem.validation_attempts >= 3 {
            AiProblem::update_status(
                &self.pool,
                problem.id,
                ProblemStatus::Rejected,
                Some("Max validation attempts exceeded"),
            )
            .await
            .map_err(|e| e.to_string())?;
            tracing::warn!("Problem {} rejected after max attempts", problem.problem_id);
        } else {
            // Reset to pending for next attempt
            AiProblem::update_status(
                &self.pool,
                problem.id,
                ProblemStatus::PendingValidation,
                Some("Pending re-validation"),
            )
            .await
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Get current pool status
    pub async fn get_status(&self) -> Result<GeneratorStatus, String> {
        let counts = AiProblem::get_pool_counts(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(GeneratorStatus {
            is_running: *self.is_running.read().await,
            pool_counts: counts,
            min_pool_easy: self.config.ai_min_pool_easy,
            min_pool_medium: self.config.ai_min_pool_medium,
            min_pool_hard: self.config.ai_min_pool_hard,
        })
    }
}

/// Status of the problem generator
#[derive(Debug, Clone)]
pub struct GeneratorStatus {
    pub is_running: bool,
    pub pool_counts: PoolCounts,
    pub min_pool_easy: u32,
    pub min_pool_medium: u32,
    pub min_pool_hard: u32,
}
