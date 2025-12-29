import { Link, useLocation } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext.tsx';
import Logo from './Logo.tsx';

export default function NavBar() {
    const { user, isAuthenticated, isLoading, login, logout } = useAuth();
    const location = useLocation();

    const isActive = (path: string) => location.pathname === path;

    return (
        <nav className="bg-zinc-800 border-b border-zinc-700">
            <div className="max-w-5xl mx-auto px-4">
                <div className="flex items-center justify-between h-12">
                    {/* Logo */}
                    <Link to="/" className="hover:opacity-80 transition-opacity">
                        <Logo size="sm" showText={true} />
                    </Link>

                    {/* Navigation Links */}
                    <div className="flex items-center gap-1">
                        <Link
                            to="/"
                            className={`px-3 py-1.5 text-sm transition-colors ${
                                isActive('/')
                                    ? 'text-white'
                                    : 'text-zinc-400 hover:text-white'
                            }`}
                        >
                            Play
                        </Link>
                        <Link
                            to="/leaderboard"
                            className={`px-3 py-1.5 text-sm transition-colors ${
                                isActive('/leaderboard')
                                    ? 'text-white'
                                    : 'text-zinc-400 hover:text-white'
                            }`}
                        >
                            Leaderboard
                        </Link>
                        {isAuthenticated && (
                            <Link
                                to="/profile"
                                className={`px-3 py-1.5 text-sm transition-colors ${
                                    isActive('/profile')
                                        ? 'text-white'
                                        : 'text-zinc-400 hover:text-white'
                                }`}
                            >
                                Profile
                            </Link>
                        )}

                        {/* Divider */}
                        <div className="w-px h-4 bg-zinc-600 mx-2" />

                        {/* Auth */}
                        {isLoading ? (
                            <div className="w-6 h-6 rounded-full bg-zinc-700 animate-pulse" />
                        ) : isAuthenticated && user ? (
                            <div className="flex items-center gap-2">
                                <Link to="/profile" className="flex items-center gap-2">
                                    {user.avatar_url ? (
                                        <img
                                            src={user.avatar_url}
                                            alt={user.display_name}
                                            className="w-6 h-6 rounded-full"
                                        />
                                    ) : (
                                        <div className="w-6 h-6 rounded-full bg-zinc-600 flex items-center justify-center text-white text-xs font-medium">
                                            {user.display_name[0].toUpperCase()}
                                        </div>
                                    )}
                                    <span className="text-white text-sm hidden sm:block">
                                        {user.display_name}
                                    </span>
                                </Link>
                                <button
                                    onClick={logout}
                                    className="text-zinc-500 hover:text-zinc-300 text-sm transition-colors"
                                >
                                    Logout
                                </button>
                            </div>
                        ) : (
                            <button
                                onClick={login}
                                className="text-zinc-400 hover:text-white text-sm transition-colors"
                            >
                                Sign In
                            </button>
                        )}
                    </div>
                </div>
            </div>
        </nav>
    );
}
