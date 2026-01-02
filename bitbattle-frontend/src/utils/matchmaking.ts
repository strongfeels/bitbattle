import { apiFetch } from './api';

export type QueueDifficulty = 'easy' | 'medium' | 'hard' | 'any';
export type GameMode = 'casual' | 'ranked';

export interface JoinQueueRequest {
    username: string;
    difficulty: QueueDifficulty;
    game_mode: GameMode;
    connection_id: string;
}

export interface JoinQueueResponse {
    success: boolean;
    message: string;
    queue_size: number;
}

export interface LeaveQueueRequest {
    connection_id: string;
}

export interface LeaveQueueResponse {
    success: boolean;
    message: string;
}

export interface MatchInfo {
    room_code: string;
    opponent: string;
    difficulty: string;
    game_mode: string;
}

export interface MatchmakingStatusResponse {
    in_queue: boolean;
    position: number | null;
    queue_size: number;
    match_found: boolean;
    match_info: MatchInfo | null;
}

// Generate a unique connection ID for this session
export function generateConnectionId(): string {
    return `conn_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
}

// Join the matchmaking queue
export async function joinQueue(request: JoinQueueRequest): Promise<JoinQueueResponse> {
    return apiFetch<JoinQueueResponse>('/matchmaking/join', {
        method: 'POST',
        body: JSON.stringify(request),
    });
}

// Leave the matchmaking queue
export async function leaveQueue(connectionId: string): Promise<LeaveQueueResponse> {
    return apiFetch<LeaveQueueResponse>('/matchmaking/leave', {
        method: 'POST',
        body: JSON.stringify({ connection_id: connectionId }),
    });
}

// Get matchmaking status
export async function getMatchmakingStatus(connectionId: string): Promise<MatchmakingStatusResponse> {
    return apiFetch<MatchmakingStatusResponse>(`/matchmaking/status?connection_id=${encodeURIComponent(connectionId)}`);
}
