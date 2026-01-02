import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import type { UserProfile, GameHistoryEntry } from '../types/auth';
import { useAuth } from '../contexts/AuthContext';
import { apiFetch } from '../utils/api';
import NavBar from './NavBar.tsx';
import Footer from './Footer.tsx';

import { ProfileSkeleton } from './Skeleton.tsx';
export default function Profile() {
    const { userId } = useParams<{ userId?: string }>();
    const { user: currentUser, isAuthenticated } = useAuth();
    const [profile, setProfile] = useState<UserProfile | null>(null);
    const [history, setHistory] = useState<GameHistoryEntry[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const targetUserId = userId || currentUser?.id;
    const isOwnProfile = !userId || userId === currentUser?.id;

    useEffect(() => {
        if (!targetUserId) {
            setIsLoading(false);
            return;
        }

        const fetchProfile = async () => {
            setIsLoading(true);
            setError(null);
            try {
                const [profileData, historyData] = await Promise.all([
                    apiFetch<UserProfile>(`/users/${targetUserId}/profile`),
                    apiFetch<GameHistoryEntry[]>(`/users/${targetUserId}/history?limit=20`),
                ]);
                setProfile(profileData);
                setHistory(historyData);
            } catch (err) {
                setError('Failed to load profile');
                console.error(err);
            } finally {
                setIsLoading(false);
            }
        };

        fetchProfile();
    }, [targetUserId]);

    const formatTime = (ms: number | null) => {
        if (ms === null) return '-';
        if (ms < 1000) return `${ms}ms`;
        return `${(ms / 1000).toFixed(1)}s`;
    };

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
        });
    };

    if (!targetUserId && !isAuthenticated) {
        return (
            <div className="min-h-screen bg-zinc-900 flex flex-col">
                <NavBar />
                <div className="flex-1 flex items-center justify-center p-4">
                    <div className="text-center">
                        <h2 className="text-xl font-semibold text-white mb-2">Sign in to view your profile</h2>
                        <p className="text-zinc-400 mb-4 text-sm">Track your stats and game history</p>
                        <Link to="/" className="text-blue-400 hover:text-blue-300 text-sm">
                            Go to Lobby
                        </Link>
                    </div>
                </div>
                <Footer />
            </div>
        );
    }

    if (isLoading) {
        return (
            <div className="min-h-screen bg-zinc-900 flex flex-col">
                <NavBar />
                <div className="flex-1 flex items-center justify-center">
                    <ProfileSkeleton />
                </div>
                <Footer />
            </div>
        );
    }

    if (error || !profile) {
        return (
            <div className="min-h-screen bg-zinc-900 flex flex-col">
                <NavBar />
                <div className="flex-1 flex items-center justify-center p-4">
                    <div className="text-center">
                        <h2 className="text-xl font-semibold text-red-400 mb-2">Profile not found</h2>
                        <Link to="/" className="text-zinc-400 hover:text-white text-sm">
                            Back to Lobby
                        </Link>
                    </div>
                </div>
                <Footer />
            </div>
        );
    }

    const winRate = profile.stats.games_played > 0
        ? ((profile.stats.games_won / profile.stats.games_played) * 100).toFixed(0)
        : 0;

    return (
        <div className="min-h-screen bg-zinc-900 flex flex-col">
            <NavBar />
            <div className="flex-1 p-4">
                <div className="max-w-3xl mx-auto space-y-4">
                    {/* Header */}
                    <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-4">
                        <div className="flex items-center gap-4">
                            {profile.avatar_url ? (
                                <img
                                    src={profile.avatar_url}
                                    alt={profile.display_name}
                                    className="w-16 h-16 rounded-full"
                                />
                            ) : (
                                <div className="w-16 h-16 rounded-full bg-zinc-700 flex items-center justify-center text-white text-2xl font-medium">
                                    {profile.display_name[0].toUpperCase()}
                                </div>
                            )}
                            <div>
                                <h1 className="text-xl font-semibold text-white">{profile.display_name}</h1>
                                {isOwnProfile && (
                                    <p className="text-zinc-500 text-sm">{profile.email}</p>
                                )}
                            </div>
                        </div>
                    </div>

                    {/* Overall Stats */}
                    <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-4">
                        <h2 className="text-sm font-medium text-zinc-400 uppercase tracking-wide mb-3">Overall</h2>
                        <div className="grid grid-cols-4 gap-4 text-center">
                            <div>
                                <div className="text-2xl font-semibold text-white">{profile.stats.games_played}</div>
                                <div className="text-xs text-zinc-500">Games</div>
                            </div>
                            <div>
                                <div className="text-2xl font-semibold text-green-400">{profile.stats.games_won}</div>
                                <div className="text-xs text-zinc-500">Wins</div>
                            </div>
                            <div>
                                <div className="text-2xl font-semibold text-white">{winRate}%</div>
                                <div className="text-xs text-zinc-500">Win Rate</div>
                            </div>
                            <div>
                                <div className="text-2xl font-semibold text-amber-400">{profile.stats.longest_streak}</div>
                                <div className="text-xs text-zinc-500">Streak</div>
                            </div>
                        </div>
                    </div>

                    {/* Ranked Ratings */}
                    <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-4">
                        <h2 className="text-sm font-medium text-zinc-400 uppercase tracking-wide mb-3">Ranked Ratings</h2>
                        <div className="space-y-3">
                            {/* Easy */}
                            <div className="flex items-center justify-between py-2 border-b border-zinc-700">
                                <div className="flex items-center gap-3">
                                    <span className="w-16 text-sm font-medium text-green-400">Easy</span>
                                    <span className="text-xl font-semibold text-white">{profile.stats.easy_ranked.rating}</span>
                                    <span className="text-xs text-zinc-500">peak {profile.stats.easy_ranked.peak_rating}</span>
                                </div>
                                <div className="text-sm text-zinc-400">
                                    {profile.stats.easy_ranked.games_won}W / {profile.stats.easy_ranked.games_played - profile.stats.easy_ranked.games_won}L
                                    {profile.stats.easy_ranked.games_played > 0 && (
                                        <span className="ml-2 text-zinc-500">({profile.stats.easy_ranked.win_rate.toFixed(0)}%)</span>
                                    )}
                                </div>
                            </div>
                            {/* Medium */}
                            <div className="flex items-center justify-between py-2 border-b border-zinc-700">
                                <div className="flex items-center gap-3">
                                    <span className="w-16 text-sm font-medium text-yellow-400">Medium</span>
                                    <span className="text-xl font-semibold text-white">{profile.stats.medium_ranked.rating}</span>
                                    <span className="text-xs text-zinc-500">peak {profile.stats.medium_ranked.peak_rating}</span>
                                </div>
                                <div className="text-sm text-zinc-400">
                                    {profile.stats.medium_ranked.games_won}W / {profile.stats.medium_ranked.games_played - profile.stats.medium_ranked.games_won}L
                                    {profile.stats.medium_ranked.games_played > 0 && (
                                        <span className="ml-2 text-zinc-500">({profile.stats.medium_ranked.win_rate.toFixed(0)}%)</span>
                                    )}
                                </div>
                            </div>
                            {/* Hard */}
                            <div className="flex items-center justify-between py-2">
                                <div className="flex items-center gap-3">
                                    <span className="w-16 text-sm font-medium text-red-400">Hard</span>
                                    <span className="text-xl font-semibold text-white">{profile.stats.hard_ranked.rating}</span>
                                    <span className="text-xs text-zinc-500">peak {profile.stats.hard_ranked.peak_rating}</span>
                                </div>
                                <div className="text-sm text-zinc-400">
                                    {profile.stats.hard_ranked.games_won}W / {profile.stats.hard_ranked.games_played - profile.stats.hard_ranked.games_won}L
                                    {profile.stats.hard_ranked.games_played > 0 && (
                                        <span className="ml-2 text-zinc-500">({profile.stats.hard_ranked.win_rate.toFixed(0)}%)</span>
                                    )}
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Recent Games */}
                    <div className="bg-zinc-800 border border-zinc-700 rounded-lg overflow-hidden">
                        <div className="px-4 py-3 border-b border-zinc-700">
                            <h2 className="text-sm font-medium text-zinc-400 uppercase tracking-wide">Recent Games</h2>
                        </div>
                        {history.length === 0 ? (
                            <div className="p-6 text-center text-zinc-500 text-sm">No games played yet</div>
                        ) : (
                            <table className="w-full text-sm">
                                <thead className="bg-zinc-900/50">
                                    <tr className="text-zinc-500 text-xs uppercase">
                                        <th className="px-4 py-2 text-left font-medium">Problem</th>
                                        <th className="px-4 py-2 text-center font-medium">Result</th>
                                        <th className="px-4 py-2 text-center font-medium">Time</th>
                                        <th className="px-4 py-2 text-center font-medium hidden sm:table-cell">Lang</th>
                                        <th className="px-4 py-2 text-right font-medium">Date</th>
                                    </tr>
                                </thead>
                                <tbody className="divide-y divide-zinc-700">
                                    {history.map((game) => (
                                        <tr key={game.id} className="hover:bg-zinc-700/30">
                                            <td className="px-4 py-2 text-white">{game.problem_id}</td>
                                            <td className="px-4 py-2 text-center">
                                                <span className={
                                                    game.placement === 1 ? 'text-yellow-400' :
                                                    game.placement === 2 ? 'text-zinc-300' :
                                                    game.placement === 3 ? 'text-amber-600' :
                                                    'text-zinc-400'
                                                }>
                                                    #{game.placement}
                                                </span>
                                                <span className="text-zinc-600 mx-1">/</span>
                                                <span className="text-zinc-500">{game.total_players}</span>
                                            </td>
                                            <td className="px-4 py-2 text-center text-zinc-300 font-mono">
                                                {formatTime(game.solve_time_ms)}
                                            </td>
                                            <td className="px-4 py-2 text-center text-zinc-500 hidden sm:table-cell">
                                                {game.language}
                                            </td>
                                            <td className="px-4 py-2 text-right text-zinc-500">
                                                {formatDate(game.created_at)}
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        )}
                    </div>
                </div>
            </div>
            <Footer />
        </div>
    );
}
