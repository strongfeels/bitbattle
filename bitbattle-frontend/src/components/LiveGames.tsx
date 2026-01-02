import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { apiFetch } from '../utils/api';
import NavBar from './NavBar.tsx';
import Footer from './Footer.tsx';

interface LiveGame {
    room_id: string;
    players: string[];
    player_count: number;
    spectator_count: number;
    game_mode: string;
    problem: {
        title: string;
        difficulty: string;
    } | null;
    game_ended: boolean;
    elapsed_seconds: number;
}

interface LiveGamesResponse {
    live_games: LiveGame[];
    total: number;
}

function formatElapsedTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
}

function getDifficultyColor(difficulty: string): string {
    switch (difficulty?.toLowerCase()) {
        case 'easy': return 'text-green-400 bg-green-400/10';
        case 'medium': return 'text-yellow-400 bg-yellow-400/10';
        case 'hard': return 'text-red-400 bg-red-400/10';
        default: return 'text-zinc-400 bg-zinc-400/10';
    }
}

export default function LiveGames() {
    const [games, setGames] = useState<LiveGame[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const fetchLiveGames = async () => {
        try {
            const data = await apiFetch<LiveGamesResponse>('/rooms/live');
            setGames(data.live_games.filter(g => !g.game_ended));
            setError(null);
        } catch (err) {
            setError('Failed to load live games');
            console.error(err);
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        fetchLiveGames();
        // Poll for updates every 5 seconds
        const interval = setInterval(fetchLiveGames, 5000);
        return () => clearInterval(interval);
    }, []);

    return (
        <div className="min-h-screen bg-zinc-900 flex flex-col">
            <NavBar />
            <div className="flex-1 p-4">
                <div className="max-w-4xl mx-auto">
                    {/* Header */}
                    <div className="mb-6">
                        <div className="flex items-center gap-3 mb-2">
                            <div className="w-3 h-3 bg-red-500 rounded-full animate-pulse" />
                            <h1 className="text-2xl font-bold text-white">Live Games</h1>
                        </div>
                        <p className="text-zinc-400">
                            {isLoading ? 'Loading...' : `${games.length} game${games.length !== 1 ? 's' : ''} in progress`}
                        </p>
                    </div>

                    {/* Games List */}
                    {isLoading ? (
                        <div className="space-y-3">
                            {[1, 2, 3].map(i => (
                                <div key={i} className="bg-zinc-800 border border-zinc-700 rounded-lg p-4 animate-pulse">
                                    <div className="h-5 bg-zinc-700 rounded w-1/3 mb-2" />
                                    <div className="h-4 bg-zinc-700 rounded w-1/2" />
                                </div>
                            ))}
                        </div>
                    ) : error ? (
                        <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-6 text-center">
                            <p className="text-red-400">{error}</p>
                            <button
                                onClick={fetchLiveGames}
                                className="mt-3 text-sm text-zinc-400 hover:text-white"
                            >
                                Try again
                            </button>
                        </div>
                    ) : games.length === 0 ? (
                        <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-8 text-center">
                            <div className="text-4xl mb-3">🎮</div>
                            <p className="text-zinc-400 mb-4">No live games right now</p>
                            <Link
                                to="/"
                                className="inline-block px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-medium transition-colors"
                            >
                                Start a Game
                            </Link>
                        </div>
                    ) : (
                        <div className="space-y-3">
                            {games.map((game) => (
                                <Link
                                    key={game.room_id}
                                    to={`/spectate/${game.room_id}`}
                                    className="block bg-zinc-800 border border-zinc-700 rounded-lg p-4 hover:border-zinc-600 transition-colors group"
                                >
                                    <div className="flex items-center justify-between mb-3">
                                        <div className="flex items-center gap-3">
                                            <div className="w-2 h-2 bg-red-500 rounded-full animate-pulse" />
                                            <span className="text-white font-medium">
                                                {game.players.join(' vs ')}
                                            </span>
                                        </div>
                                        <div className="flex items-center gap-2 text-sm">
                                            <span className={`px-2 py-0.5 rounded ${game.game_mode === 'ranked' ? 'text-amber-400 bg-amber-400/10' : 'text-blue-400 bg-blue-400/10'}`}>
                                                {game.game_mode}
                                            </span>
                                            {game.problem && (
                                                <span className={`px-2 py-0.5 rounded ${getDifficultyColor(game.problem.difficulty)}`}>
                                                    {game.problem.difficulty}
                                                </span>
                                            )}
                                        </div>
                                    </div>

                                    <div className="flex items-center justify-between text-sm">
                                        <div className="flex items-center gap-4 text-zinc-400">
                                            {game.problem && (
                                                <span>Problem: {game.problem.title}</span>
                                            )}
                                            <span>⏱️ {formatElapsedTime(game.elapsed_seconds)}</span>
                                        </div>
                                        <div className="flex items-center gap-2">
                                            <span className="text-zinc-500">
                                                👁️ {game.spectator_count} watching
                                            </span>
                                            <span className="text-blue-400 opacity-0 group-hover:opacity-100 transition-opacity">
                                                Watch →
                                            </span>
                                        </div>
                                    </div>
                                </Link>
                            ))}
                        </div>
                    )}

                    {/* Back link */}
                    <div className="mt-6 text-center">
                        <Link to="/" className="text-zinc-400 hover:text-white text-sm transition-colors">
                            ← Back to Lobby
                        </Link>
                    </div>
                </div>
            </div>
            <Footer />
        </div>
    );
}
