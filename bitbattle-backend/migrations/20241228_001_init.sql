-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    google_id VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    avatar_url TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User stats table for leaderboard
CREATE TABLE user_stats (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    games_played INTEGER DEFAULT 0,
    games_won INTEGER DEFAULT 0,
    games_lost INTEGER DEFAULT 0,
    problems_solved INTEGER DEFAULT 0,
    total_submissions INTEGER DEFAULT 0,
    fastest_solve_ms BIGINT,
    current_streak INTEGER DEFAULT 0,
    longest_streak INTEGER DEFAULT 0,
    last_played_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Game results table
CREATE TABLE game_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id VARCHAR(50) NOT NULL,
    problem_id VARCHAR(100) NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    placement INTEGER NOT NULL,
    total_players INTEGER NOT NULL,
    solve_time_ms BIGINT,
    passed_tests INTEGER NOT NULL,
    total_tests INTEGER NOT NULL,
    language VARCHAR(20) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for leaderboard queries
CREATE INDEX idx_user_stats_games_won ON user_stats(games_won DESC);
CREATE INDEX idx_user_stats_problems_solved ON user_stats(problems_solved DESC);
CREATE INDEX idx_user_stats_fastest_solve ON user_stats(fastest_solve_ms ASC) WHERE fastest_solve_ms IS NOT NULL;
CREATE INDEX idx_user_stats_longest_streak ON user_stats(longest_streak DESC);
CREATE INDEX idx_game_results_user_id ON game_results(user_id);
CREATE INDEX idx_game_results_created_at ON game_results(created_at DESC);
