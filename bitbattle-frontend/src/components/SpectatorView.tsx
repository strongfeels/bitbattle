import { useState, useEffect, useRef } from 'react';
import { useParams, Link } from 'react-router-dom';
import Logo from './Logo.tsx';

interface Problem {
    id: string;
    title: string;
    description: string;
    difficulty: string;
    examples: Array<{
        input: string;
        expected_output: string;
        explanation?: string;
    }>;
}

interface SpectateInitData {
    room_id: string;
    players: string[];
    game_mode: string;
    game_started: boolean;
    game_ended: boolean;
    winner: string | null;
    problem: Problem | null;
    player_codes: Record<string, string>;
    spectator_count: number;
}

interface GameOverData {
    winner: string;
    solve_time_ms: number;
    problem_id: string;
    difficulty: string;
    game_mode: string;
}

const API_WS_URL = import.meta.env.VITE_API_WS_URL || 'ws://localhost:4000';

export default function SpectatorView() {
    const { roomCode } = useParams<{ roomCode: string }>();
    const [isConnected, setIsConnected] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [players, setPlayers] = useState<string[]>([]);
    const [playerCodes, setPlayerCodes] = useState<Record<string, string>>({});
    const [problem, setProblem] = useState<Problem | null>(null);
    const [gameMode, setGameMode] = useState<string>('casual');
    const [gameEnded, setGameEnded] = useState(false);
    const [winner, setWinner] = useState<string | null>(null);
    const [spectatorCount, setSpectatorCount] = useState(0);
    const [gameOverData, setGameOverData] = useState<GameOverData | null>(null);
    const wsRef = useRef<WebSocket | null>(null);

    useEffect(() => {
        if (!roomCode) return;

        const ws = new WebSocket(`${API_WS_URL}/ws/spectate?room=${roomCode}`);
        wsRef.current = ws;

        ws.onopen = () => {
            setIsConnected(true);
            setError(null);
        };

        ws.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);

                switch (message.type) {
                    case 'spectate_init': {
                        const data = message.data as SpectateInitData;
                        setPlayers(data.players);
                        setPlayerCodes(data.player_codes);
                        setProblem(data.problem);
                        setGameMode(data.game_mode);
                        setGameEnded(data.game_ended);
                        setWinner(data.winner);
                        setSpectatorCount(data.spectator_count);
                        break;
                    }
                    case 'code_change': {
                        const { username, code } = message.data;
                        setPlayerCodes(prev => ({ ...prev, [username]: code }));
                        break;
                    }
                    case 'user_joined': {
                        const { username } = message.data;
                        setPlayers(prev => prev.includes(username) ? prev : [...prev, username]);
                        break;
                    }
                    case 'user_left': {
                        const { username } = message.data;
                        setPlayers(prev => prev.filter(p => p !== username));
                        break;
                    }
                    case 'game_over': {
                        setGameEnded(true);
                        setWinner(message.data.winner);
                        setGameOverData(message.data);
                        break;
                    }
                    case 'error': {
                        setError(message.data.message);
                        break;
                    }
                }
            } catch (err) {
                console.error('Failed to parse message:', err);
            }
        };

        ws.onerror = () => {
            setError('Connection error');
            setIsConnected(false);
        };

        ws.onclose = () => {
            setIsConnected(false);
        };

        return () => {
            ws.close();
        };
    }, [roomCode]);

    if (error) {
        return (
            <div className="min-h-screen bg-zinc-900 flex items-center justify-center p-4">
                <div className="text-center">
                    <div className="text-red-400 text-xl mb-4">{error}</div>
                    <Link
                        to="/live"
                        className="text-blue-400 hover:text-blue-300"
                    >
                        ← Back to Live Games
                    </Link>
                </div>
            </div>
        );
    }

    if (!isConnected) {
        return (
            <div className="min-h-screen bg-zinc-900 flex items-center justify-center">
                <div className="text-zinc-400">Connecting to game...</div>
            </div>
        );
    }

    return (
        <div className="h-screen flex flex-col bg-zinc-900">
            {/* Header */}
            <div className="bg-zinc-800 border-b border-zinc-700 px-4 py-2 flex justify-between items-center">
                <div className="flex items-center gap-4">
                    <Logo size="sm" showText={true} />
                    <div className="w-px h-4 bg-zinc-600" />
                    <div className="flex items-center gap-2">
                        <div className="w-2 h-2 bg-red-500 rounded-full animate-pulse" />
                        <span className="text-red-400 text-sm font-medium">LIVE</span>
                    </div>
                    <span className={`text-xs px-2 py-0.5 rounded ${gameMode === 'ranked' ? 'text-amber-400 bg-amber-400/10' : 'text-blue-400 bg-blue-400/10'}`}>
                        {gameMode}
                    </span>
                    <span className="text-zinc-500 text-sm">
                        👁️ {spectatorCount} watching
                    </span>
                </div>
                <Link
                    to="/live"
                    className="text-zinc-400 hover:text-white text-sm px-3 py-1 rounded hover:bg-zinc-700 transition-colors"
                >
                    Exit
                </Link>
            </div>

            {/* Problem Title Bar */}
            {problem && (
                <div className="bg-zinc-800/50 border-b border-zinc-700 px-4 py-2">
                    <div className="flex items-center gap-3">
                        <span className="text-white font-medium">{problem.title}</span>
                        <span className={`text-xs px-2 py-0.5 rounded ${
                            problem.difficulty === 'Easy' ? 'text-green-400 bg-green-400/10' :
                            problem.difficulty === 'Medium' ? 'text-yellow-400 bg-yellow-400/10' :
                            'text-red-400 bg-red-400/10'
                        }`}>
                            {problem.difficulty}
                        </span>
                    </div>
                </div>
            )}

            {/* Game Over Overlay */}
            {gameEnded && gameOverData && (
                <div className="absolute inset-0 bg-black/80 flex items-center justify-center z-50">
                    <div className="bg-zinc-800 border border-zinc-700 rounded-xl p-8 text-center max-w-md">
                        <div className="text-4xl mb-4">🏆</div>
                        <h2 className="text-2xl font-bold text-white mb-2">Game Over!</h2>
                        <p className="text-zinc-400 mb-4">
                            <span className="text-green-400 font-semibold">{gameOverData.winner}</span> wins!
                        </p>
                        <p className="text-zinc-500 text-sm mb-6">
                            Solved in {(gameOverData.solve_time_ms / 1000).toFixed(1)}s
                        </p>
                        <Link
                            to="/live"
                            className="inline-block px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors"
                        >
                            Back to Live Games
                        </Link>
                    </div>
                </div>
            )}

            {/* Main Content - Side by Side Code Views */}
            <div className="flex-1 flex overflow-hidden">
                {players.map((player, index) => (
                    <div
                        key={player}
                        className={`flex-1 flex flex-col ${index > 0 ? 'border-l border-zinc-700' : ''}`}
                    >
                        {/* Player Header */}
                        <div className="bg-zinc-800 border-b border-zinc-700 px-4 py-2 flex items-center justify-between">
                            <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-full bg-zinc-600 flex items-center justify-center text-white text-xs font-medium">
                                    {player[0].toUpperCase()}
                                </div>
                                <span className="text-white font-medium">{player}</span>
                                {winner === player && (
                                    <span className="text-yellow-400 text-sm">👑 Winner</span>
                                )}
                            </div>
                            <div className="text-zinc-500 text-xs">
                                {(playerCodes[player] || '').split('\n').length} lines
                            </div>
                        </div>

                        {/* Code Display */}
                        <div className="flex-1 overflow-auto bg-zinc-950 p-4">
                            <pre className="text-sm font-mono text-zinc-300 whitespace-pre-wrap">
                                {playerCodes[player] || '// Waiting for code...'}
                            </pre>
                        </div>
                    </div>
                ))}

                {players.length === 0 && (
                    <div className="flex-1 flex items-center justify-center text-zinc-500">
                        Waiting for players...
                    </div>
                )}
            </div>

            {/* Problem Description (collapsible) */}
            {problem && (
                <details className="bg-zinc-800 border-t border-zinc-700">
                    <summary className="px-4 py-2 text-zinc-400 text-sm cursor-pointer hover:text-white">
                        View Problem Description
                    </summary>
                    <div className="px-4 py-3 border-t border-zinc-700 max-h-48 overflow-auto">
                        <p className="text-zinc-300 text-sm whitespace-pre-wrap">{problem.description}</p>
                        {problem.examples.length > 0 && (
                            <div className="mt-3">
                                <h4 className="text-zinc-400 text-xs font-medium mb-2">Examples:</h4>
                                {problem.examples.map((ex, i) => (
                                    <div key={i} className="bg-zinc-900 rounded p-2 mb-2 text-xs">
                                        <div><span className="text-zinc-500">Input:</span> <code className="text-zinc-300">{ex.input}</code></div>
                                        <div><span className="text-zinc-500">Output:</span> <code className="text-zinc-300">{ex.expected_output}</code></div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </details>
            )}
        </div>
    );
}
