-- Add per-difficulty ELO ratings to user_stats
-- Easy difficulty ratings
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS easy_rating INTEGER DEFAULT 1200;
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS easy_peak_rating INTEGER DEFAULT 1200;
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS easy_ranked_games INTEGER DEFAULT 0;
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS easy_ranked_wins INTEGER DEFAULT 0;

-- Medium difficulty ratings
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS medium_rating INTEGER DEFAULT 1200;
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS medium_peak_rating INTEGER DEFAULT 1200;
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS medium_ranked_games INTEGER DEFAULT 0;
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS medium_ranked_wins INTEGER DEFAULT 0;

-- Hard difficulty ratings
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS hard_rating INTEGER DEFAULT 1200;
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS hard_peak_rating INTEGER DEFAULT 1200;
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS hard_ranked_games INTEGER DEFAULT 0;
ALTER TABLE user_stats ADD COLUMN IF NOT EXISTS hard_ranked_wins INTEGER DEFAULT 0;

-- Add game_mode and difficulty to game_results
ALTER TABLE game_results ADD COLUMN IF NOT EXISTS game_mode VARCHAR(20) DEFAULT 'casual';
ALTER TABLE game_results ADD COLUMN IF NOT EXISTS rating_change INTEGER DEFAULT 0;
ALTER TABLE game_results ADD COLUMN IF NOT EXISTS difficulty VARCHAR(20) DEFAULT 'medium';
