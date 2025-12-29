export interface User {
    id: string;
    email: string;
    display_name: string;
    avatar_url: string | null;
}

export interface DifficultyRankedStats {
    rating: number;
    peak_rating: number;
    games_played: number;
    games_won: number;
    win_rate: number;
}

export interface UserStats {
    games_played: number;
    games_won: number;
    games_lost: number;
    problems_solved: number;
    fastest_solve_ms: number | null;
    current_streak: number;
    longest_streak: number;
    easy_ranked: DifficultyRankedStats;
    medium_ranked: DifficultyRankedStats;
    hard_ranked: DifficultyRankedStats;
}

export interface ProblemBest {
    problem_id: string;
    fastest_solve_ms: number;
    attempts: number;
}

export interface UserProfile {
    id: string;
    email: string;
    display_name: string;
    avatar_url: string | null;
    stats: UserStats;
    problem_bests: ProblemBest[];
}

export interface LeaderboardEntry {
    rank: number;
    user_id: string;
    display_name: string;
    avatar_url: string | null;
    games_played: number;
    games_won: number;
    win_rate: number;
    problems_solved: number;
    fastest_solve_ms: number | null;
    longest_streak: number;
}

export interface GameHistoryEntry {
    id: string;
    room_id: string;
    problem_id: string;
    placement: number;
    total_players: number;
    solve_time_ms: number | null;
    passed_tests: number;
    total_tests: number;
    language: string;
    created_at: string;
}
