-- AI Generated Problems table
CREATE TABLE IF NOT EXISTS ai_problems (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    problem_id VARCHAR(100) UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    difficulty VARCHAR(20) NOT NULL CHECK (difficulty IN ('Easy', 'Medium', 'Hard')),
    examples JSONB NOT NULL,
    test_cases JSONB NOT NULL,
    starter_code JSONB NOT NULL,
    time_limit_minutes INTEGER,
    tags JSONB NOT NULL DEFAULT '[]',

    -- Status & metadata
    status VARCHAR(30) NOT NULL DEFAULT 'pending_validation'
        CHECK (status IN ('pending_validation', 'validating', 'validated', 'rejected')),
    provider VARCHAR(50) NOT NULL,
    model VARCHAR(100) NOT NULL,
    validation_attempts INTEGER DEFAULT 0,
    last_validation_error TEXT,
    validated_at TIMESTAMP WITH TIME ZONE,

    -- Usage tracking
    times_used INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Track which players have seen which problems (prevent repeats)
CREATE TABLE IF NOT EXISTS player_problem_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    problem_id VARCHAR(100) NOT NULL,
    played_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_ai_problems_status_difficulty ON ai_problems(status, difficulty);
CREATE INDEX IF NOT EXISTS idx_ai_problems_times_used ON ai_problems(times_used ASC) WHERE status = 'validated';
CREATE INDEX IF NOT EXISTS idx_player_history_user ON player_problem_history(user_id, problem_id);
CREATE INDEX IF NOT EXISTS idx_player_history_lookup ON player_problem_history(user_id, played_at DESC);
