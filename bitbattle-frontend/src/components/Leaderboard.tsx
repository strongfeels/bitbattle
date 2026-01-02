import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import type { LeaderboardEntry } from '../types/auth';
import { apiFetch } from '../utils/api';
import NavBar from './NavBar.tsx';
import Footer from './Footer.tsx';

import { LeaderboardSkeleton } from './Skeleton.tsx';
type SortOption = 'wins' | 'problems_solved' | 'fastest' | 'streak';

export default function Leaderboard() {
    const [entries, setEntries] = useState<LeaderboardEntry[]>([]);
    const [total, setTotal] = useState(0);
    const [isLoading, setIsLoading] = useState(true);
    const [sortBy, setSortBy] = useState<SortOption>('wins');
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const fetchLeaderboard = async () => {
            setIsLoading(true);
            setError(null);
            try {
                const data = await apiFetch<{ entries: LeaderboardEntry[]; total: number }>(
                    `/leaderboard?sort_by=${sortBy}&limit=50`
                );
                setEntries(data.entries);
                setTotal(data.total);
            } catch (err) {
                setError('Failed to load leaderboard');
                console.error(err);
            } finally {
                setIsLoading(false);
            }
        };

        fetchLeaderboard();
    }, [sortBy]);

    const formatTime = (ms: number | null) => {
        if (ms === null) return '-';
        if (ms < 1000) return `${ms}ms`;
        return `${(ms / 1000).toFixed(1)}s`;
    };

    return (
        <div className="min-h-screen bg-gradient-to-br from-purple-900 via-indigo-900 to-blue-900 flex flex-col">
            <NavBar />
            <div className="flex-1 p-4">
                <div className="max-w-4xl mx-auto">
                    {/* Header */}
                    <div className="mb-6">
                        <h1 className="text-3xl font-bold text-white">Leaderboard</h1>
                        <p className="text-purple-200">{total} players ranked</p>
                    </div>

                    {/* Sort Options */}
                    <div className="bg-white/10 backdrop-blur-md rounded-xl p-4 mb-6">
                        <div className="flex flex-wrap gap-2">
                            <span className="text-white/70 text-sm self-center mr-2">Sort by:</span>
                            {[
                                { value: 'wins', label: 'Most Wins' },
                                { value: 'problems_solved', label: 'Problems Solved' },
                                { value: 'fastest', label: 'Fastest Solve' },
                                { value: 'streak', label: 'Longest Streak' },
                            ].map((option) => (
                                <button
                                    key={option.value}
                                    onClick={() => setSortBy(option.value as SortOption)}
                                    className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-all ${
                                        sortBy === option.value
                                            ? 'bg-purple-600 text-white'
                                            : 'bg-white/10 text-white/70 hover:bg-white/20'
                                    }`}
                                >
                                    {option.label}
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* Leaderboard Table */}
                    <div className="bg-white/10 backdrop-blur-md rounded-xl overflow-hidden">
                        {isLoading ? (
                            <LeaderboardSkeleton />
                        ) : error ? (
                            <div className="p-8 text-center text-red-300">{error}</div>
                        ) : entries.length === 0 ? (
                            <div className="p-8 text-center text-white/70">No players on the leaderboard yet</div>
                        ) : (
                            <div className="overflow-x-auto">
                                <table className="w-full">
                                    <thead className="bg-white/10">
                                        <tr>
                                            <th className="px-4 py-3 text-left text-sm font-medium text-white/70">Rank</th>
                                            <th className="px-4 py-3 text-left text-sm font-medium text-white/70">Player</th>
                                            <th className="px-4 py-3 text-center text-sm font-medium text-white/70">Games</th>
                                            <th className="px-4 py-3 text-center text-sm font-medium text-white/70">Wins</th>
                                            <th className="px-4 py-3 text-center text-sm font-medium text-white/70">Win %</th>
                                            <th className="px-4 py-3 text-center text-sm font-medium text-white/70">Solved</th>
                                            <th className="px-4 py-3 text-center text-sm font-medium text-white/70">Fastest</th>
                                            <th className="px-4 py-3 text-center text-sm font-medium text-white/70">Streak</th>
                                        </tr>
                                    </thead>
                                    <tbody className="divide-y divide-white/10">
                                        {entries.map((entry) => (
                                            <tr key={entry.user_id} className="hover:bg-white/5 transition-colors">
                                                <td className="px-4 py-3">
                                                    <span className={`font-bold ${
                                                        entry.rank === 1 ? 'text-yellow-400' :
                                                        entry.rank === 2 ? 'text-gray-300' :
                                                        entry.rank === 3 ? 'text-amber-600' :
                                                        'text-white/70'
                                                    }`}>
                                                        {entry.rank === 1 && '🥇 '}
                                                        {entry.rank === 2 && '🥈 '}
                                                        {entry.rank === 3 && '🥉 '}
                                                        #{entry.rank}
                                                    </span>
                                                </td>
                                                <td className="px-4 py-3">
                                                    <Link
                                                        to={`/profile/${entry.user_id}`}
                                                        className="flex items-center space-x-3 hover:opacity-80"
                                                    >
                                                        {entry.avatar_url ? (
                                                            <img
                                                                src={entry.avatar_url}
                                                                alt={entry.display_name}
                                                                className="w-8 h-8 rounded-full"
                                                            />
                                                        ) : (
                                                            <div className="w-8 h-8 rounded-full bg-purple-600 flex items-center justify-center text-white font-bold">
                                                                {entry.display_name[0].toUpperCase()}
                                                            </div>
                                                        )}
                                                        <span className="text-white font-medium">{entry.display_name}</span>
                                                    </Link>
                                                </td>
                                                <td className="px-4 py-3 text-center text-white/70">{entry.games_played}</td>
                                                <td className="px-4 py-3 text-center text-green-400 font-medium">{entry.games_won}</td>
                                                <td className="px-4 py-3 text-center text-white/70">{entry.win_rate.toFixed(1)}%</td>
                                                <td className="px-4 py-3 text-center text-blue-400">{entry.problems_solved}</td>
                                                <td className="px-4 py-3 text-center text-yellow-400">{formatTime(entry.fastest_solve_ms)}</td>
                                                <td className="px-4 py-3 text-center text-orange-400">{entry.longest_streak}</td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                        )}
                    </div>
                </div>
            </div>
            <Footer />
        </div>
    );
}
