import { useState, useEffect, useCallback, useRef } from 'react';
import {
    joinQueue,
    leaveQueue,
    getMatchmakingStatus,
    generateConnectionId,
    type QueueDifficulty,
    type GameMode,
    type MatchInfo,
} from '../utils/matchmaking';

interface Props {
    username: string;
    difficulty: QueueDifficulty;
    gameMode: GameMode;
    onMatchFound: (matchInfo: MatchInfo) => void;
    onCancel: () => void;
}

type QueueState = 'joining' | 'queued' | 'match_found' | 'error';

export default function MatchmakingQueue({ username, difficulty, gameMode, onMatchFound, onCancel }: Props) {
    const [state, setState] = useState<QueueState>('joining');
    const [queuePosition, setQueuePosition] = useState<number | null>(null);
    const [queueSize, setQueueSize] = useState(0);
    const [elapsedTime, setElapsedTime] = useState(0);
    const [error, setError] = useState<string | null>(null);
    const [matchInfo, setMatchInfo] = useState<MatchInfo | null>(null);

    const connectionIdRef = useRef<string>(generateConnectionId());
    const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null);
    const timerIntervalRef = useRef<NodeJS.Timeout | null>(null);

    // Join queue on mount
    useEffect(() => {
        const joinMatchmaking = async () => {
            try {
                const response = await joinQueue({
                    username,
                    difficulty,
                    game_mode: gameMode,
                    connection_id: connectionIdRef.current,
                });

                if (response.success) {
                    setState('queued');
                    setQueueSize(response.queue_size);
                } else {
                    setState('error');
                    setError(response.message);
                }
            } catch (err) {
                setState('error');
                setError('Failed to join matchmaking queue');
                console.error('Join queue error:', err);
            }
        };

        joinMatchmaking();

        // Cleanup on unmount
        return () => {
            if (pollingIntervalRef.current) {
                clearInterval(pollingIntervalRef.current);
            }
            if (timerIntervalRef.current) {
                clearInterval(timerIntervalRef.current);
            }
            // Leave queue on unmount
            leaveQueue(connectionIdRef.current).catch(console.error);
        };
    }, [username, difficulty, gameMode]);

    // Poll for match status
    useEffect(() => {
        if (state !== 'queued') return;

        const checkStatus = async () => {
            try {
                const status = await getMatchmakingStatus(connectionIdRef.current);

                if (status.match_found && status.match_info) {
                    setState('match_found');
                    setMatchInfo(status.match_info);

                    // Clear intervals
                    if (pollingIntervalRef.current) {
                        clearInterval(pollingIntervalRef.current);
                    }
                    if (timerIntervalRef.current) {
                        clearInterval(timerIntervalRef.current);
                    }

                    // Notify parent after a short delay for animation
                    setTimeout(() => {
                        onMatchFound(status.match_info!);
                    }, 1500);
                } else {
                    setQueuePosition(status.position);
                    setQueueSize(status.queue_size);
                }
            } catch (err) {
                console.error('Status check error:', err);
            }
        };

        // Initial check
        checkStatus();

        // Poll every 2 seconds
        pollingIntervalRef.current = setInterval(checkStatus, 2000);

        return () => {
            if (pollingIntervalRef.current) {
                clearInterval(pollingIntervalRef.current);
            }
        };
    }, [state, onMatchFound]);

    // Timer
    useEffect(() => {
        if (state !== 'queued') return;

        timerIntervalRef.current = setInterval(() => {
            setElapsedTime((prev) => prev + 1);
        }, 1000);

        return () => {
            if (timerIntervalRef.current) {
                clearInterval(timerIntervalRef.current);
            }
        };
    }, [state]);

    const handleCancel = useCallback(async () => {
        try {
            await leaveQueue(connectionIdRef.current);
        } catch (err) {
            console.error('Leave queue error:', err);
        }
        onCancel();
    }, [onCancel]);

    const formatTime = (seconds: number): string => {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    };

    const getDifficultyColor = (diff: string): string => {
        switch (diff.toLowerCase()) {
            case 'easy':
                return 'text-green-400';
            case 'medium':
                return 'text-yellow-400';
            case 'hard':
                return 'text-red-400';
            default:
                return 'text-purple-400';
        }
    };

    const getDifficultyLabel = (diff: QueueDifficulty): string => {
        return diff === 'any' ? 'Any' : diff.charAt(0).toUpperCase() + diff.slice(1);
    };

    if (state === 'error') {
        return (
            <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50">
                <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-6 max-w-sm w-full mx-4">
                    <div className="text-center">
                        <div className="w-12 h-12 rounded-full bg-red-500/20 flex items-center justify-center mx-auto mb-4">
                            <svg className="w-6 h-6 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </div>
                        <h3 className="text-lg font-semibold text-white mb-2">Error</h3>
                        <p className="text-zinc-400 text-sm mb-4">{error}</p>
                        <button
                            onClick={onCancel}
                            className="w-full py-2 rounded text-sm font-medium bg-zinc-700 text-white hover:bg-zinc-600 transition-colors"
                        >
                            Go Back
                        </button>
                    </div>
                </div>
            </div>
        );
    }

    if (state === 'match_found' && matchInfo) {
        return (
            <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50">
                <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-6 max-w-sm w-full mx-4">
                    <div className="text-center">
                        <div className="w-16 h-16 rounded-full bg-green-500/20 flex items-center justify-center mx-auto mb-4 animate-pulse">
                            <svg className="w-8 h-8 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                            </svg>
                        </div>
                        <h3 className="text-xl font-bold text-white mb-2">Match Found!</h3>
                        <div className="space-y-2 mb-4">
                            <p className="text-zinc-300">
                                vs <span className="font-semibold text-white">{matchInfo.opponent}</span>
                            </p>
                            <p className="text-sm text-zinc-400">
                                <span className={getDifficultyColor(matchInfo.difficulty)}>
                                    {matchInfo.difficulty.charAt(0).toUpperCase() + matchInfo.difficulty.slice(1)}
                                </span>
                                {' · '}
                                <span className={matchInfo.game_mode === 'ranked' ? 'text-amber-400' : 'text-blue-400'}>
                                    {matchInfo.game_mode.charAt(0).toUpperCase() + matchInfo.game_mode.slice(1)}
                                </span>
                            </p>
                        </div>
                        <div className="flex items-center justify-center gap-2 text-zinc-500 text-sm">
                            <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                            </svg>
                            Joining room...
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50">
            <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-6 max-w-sm w-full mx-4">
                <div className="text-center">
                    {/* Searching animation */}
                    <div className="relative w-20 h-20 mx-auto mb-4">
                        <div className="absolute inset-0 rounded-full border-4 border-zinc-700" />
                        <div className="absolute inset-0 rounded-full border-4 border-transparent border-t-blue-500 animate-spin" />
                        <div className="absolute inset-2 rounded-full border-4 border-transparent border-t-blue-400 animate-spin" style={{ animationDuration: '1.5s', animationDirection: 'reverse' }} />
                        <div className="absolute inset-0 flex items-center justify-center">
                            <span className="text-2xl font-bold text-white">{formatTime(elapsedTime)}</span>
                        </div>
                    </div>

                    <h3 className="text-lg font-semibold text-white mb-1">
                        {state === 'joining' ? 'Joining Queue...' : 'Finding Match...'}
                    </h3>

                    {/* Queue info */}
                    <div className="space-y-1 mb-4">
                        <p className="text-zinc-400 text-sm">
                            <span className={getDifficultyColor(difficulty)}>
                                {getDifficultyLabel(difficulty)}
                            </span>
                            {' · '}
                            <span className={gameMode === 'ranked' ? 'text-amber-400' : 'text-blue-400'}>
                                {gameMode.charAt(0).toUpperCase() + gameMode.slice(1)}
                            </span>
                        </p>
                        {queuePosition !== null && (
                            <p className="text-zinc-500 text-xs">
                                Position in queue: {queuePosition + 1} of {queueSize}
                            </p>
                        )}
                    </div>

                    {/* Players in queue indicator */}
                    <div className="bg-zinc-900 rounded-lg p-3 mb-4">
                        <div className="flex items-center justify-between text-sm">
                            <span className="text-zinc-500">Players searching</span>
                            <span className="text-white font-medium">{queueSize}</span>
                        </div>
                        <div className="mt-2 flex items-center gap-1">
                            {[...Array(Math.min(queueSize, 8))].map((_, i) => (
                                <div
                                    key={i}
                                    className="w-2 h-2 rounded-full bg-green-500 animate-pulse"
                                    style={{ animationDelay: `${i * 0.15}s` }}
                                />
                            ))}
                            {queueSize > 8 && (
                                <span className="text-zinc-500 text-xs ml-1">+{queueSize - 8}</span>
                            )}
                        </div>
                    </div>

                    {/* Tips */}
                    <p className="text-zinc-600 text-xs mb-4">
                        {elapsedTime < 30
                            ? 'Searching for players with similar rating...'
                            : elapsedTime < 60
                            ? 'Expanding search range...'
                            : 'Searching all available players...'}
                    </p>

                    <button
                        onClick={handleCancel}
                        className="w-full py-2 rounded text-sm font-medium border border-zinc-600 text-zinc-300 hover:bg-zinc-700 transition-colors"
                    >
                        Cancel
                    </button>
                </div>
            </div>
        </div>
    );
}
