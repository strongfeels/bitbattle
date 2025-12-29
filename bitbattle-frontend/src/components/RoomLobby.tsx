import { useState, useEffect } from 'react';
import { generateRoomCode, isValidRoomCode, formatRoomCode } from '../utils/roomUtils.ts';
import { useAuth } from '../contexts/AuthContext.tsx';
import GoogleLoginButton from './GoogleLoginButton.tsx';
import Logo from './Logo.tsx';
import NavBar from './NavBar.tsx';
import Footer from './Footer.tsx';

type Difficulty = 'random' | 'easy' | 'medium' | 'hard';
type GameMode = 'casual' | 'ranked';

interface Props {
    onJoinRoom: (roomCode: string, username: string, difficulty?: Difficulty, playerCount?: number, gameMode?: GameMode) => void;
    username: string;
    setUsername: (username: string) => void;
}

// Generate a random guest name
function generateGuestName(): string {
    const num = Math.floor(Math.random() * 10000).toString().padStart(4, '0');
    return `guest_${num}`;
}

export default function RoomLobby({ onJoinRoom, username, setUsername }: Props) {
    const { user, isAuthenticated, isLoading } = useAuth();
    const [mode, setMode] = useState<'home' | 'create' | 'join'>('home');
    const [roomCode, setRoomCode] = useState('');
    const [isCreating, setIsCreating] = useState(false);
    const [difficulty, setDifficulty] = useState<Difficulty>('random');
    const [playerCount, setPlayerCount] = useState<number>(2);
    const [gameMode, setGameMode] = useState<GameMode>('casual');

    // Set username based on auth status
    useEffect(() => {
        if (isAuthenticated && user) {
            setUsername(user.display_name);
        } else if (!isLoading && !isAuthenticated) {
            // Generate guest name when not authenticated (including after logout)
            if (!username || !username.startsWith('guest_')) {
                setUsername(generateGuestName());
            }
        }
    }, [isAuthenticated, user, isLoading, username, setUsername]);

    const displayName = isAuthenticated && user ? user.display_name : username;

    const handleCreateRoom = async () => {
        if (gameMode === 'ranked' && !isAuthenticated) {
            return;
        }

        setIsCreating(true);
        const newRoomCode = generateRoomCode();

        setTimeout(() => {
            setIsCreating(false);
            onJoinRoom(newRoomCode, displayName, difficulty, playerCount, gameMode);
        }, 500);
    };

    const handleJoinRoom = () => {
        if (!roomCode.trim()) return;

        const formattedCode = formatRoomCode(roomCode.trim());
        if (!isValidRoomCode(formattedCode)) {
            alert('Please enter a valid room code (e.g., SWIFT-CODER-1234)');
            return;
        }

        onJoinRoom(formattedCode, displayName);
    };

    const handleQuickJoin = () => {
        onJoinRoom('QUICK-BATTLE-0001', displayName);
    };

    if (mode === 'home') {
        return (
            <div className="min-h-screen bg-gradient-to-b from-zinc-800 via-zinc-900 to-zinc-950 flex flex-col">
                <NavBar />
                <div className="flex-1 flex items-center justify-center p-4">
                    <div className="w-full max-w-sm">
                        {/* Header */}
                        <div className="text-center mb-8">
                            <div className="flex justify-center mb-4">
                                <Logo size="xl" />
                            </div>
                            <h1 className="text-3xl font-bold text-white mb-1">BitBattle</h1>
                            <p className="text-zinc-400 text-sm">Competitive coding arena</p>
                        </div>

                        {/* Auth or Guest Display */}
                        <div className="mb-6">
                            {isLoading ? (
                                <div className="text-center text-zinc-500 text-sm">Loading...</div>
                            ) : isAuthenticated && user ? (
                                <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-3 text-center">
                                    <span className="text-zinc-400 text-sm">Playing as </span>
                                    <span className="text-white font-medium">{user.display_name}</span>
                                </div>
                            ) : (
                                <div className="space-y-3">
                                    <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-3 text-center">
                                        <span className="text-zinc-400 text-sm">Playing as </span>
                                        <span className="text-zinc-300 font-mono">{username}</span>
                                    </div>
                                    <GoogleLoginButton />
                                </div>
                            )}
                        </div>

                        {/* Action Buttons */}
                        <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-5 space-y-2">
                            <button
                                onClick={() => setMode('create')}
                                className="w-full py-2.5 rounded text-sm font-medium bg-green-600 hover:bg-green-500 text-white transition-colors"
                            >
                                Create Room
                            </button>

                            <button
                                onClick={() => setMode('join')}
                                className="w-full py-2.5 rounded text-sm font-medium bg-blue-600 hover:bg-blue-500 text-white transition-colors"
                            >
                                Join Room
                            </button>

                            <button
                                onClick={handleQuickJoin}
                                className="w-full py-2.5 rounded text-sm font-medium border border-zinc-600 text-zinc-300 hover:bg-zinc-700 transition-colors"
                            >
                                Quick Match
                            </button>
                        </div>

                        {!isAuthenticated && (
                            <p className="text-center text-zinc-500 text-xs mt-4">
                                Sign in to save stats and play ranked
                            </p>
                        )}

                        {/* How it works */}
                        <div className="mt-10 text-center">
                            <h2 className="text-zinc-400 text-xs uppercase tracking-wide mb-4">How it works</h2>
                            <div className="space-y-3 text-sm">
                                <div className="flex items-start gap-3 text-left">
                                    <span className="text-zinc-500 font-mono">1.</span>
                                    <p className="text-zinc-400">Create a room and share the code with friends, or join an existing room</p>
                                </div>
                                <div className="flex items-start gap-3 text-left">
                                    <span className="text-zinc-500 font-mono">2.</span>
                                    <p className="text-zinc-400">Once all players join, a coding problem appears and the timer starts</p>
                                </div>
                                <div className="flex items-start gap-3 text-left">
                                    <span className="text-zinc-500 font-mono">3.</span>
                                    <p className="text-zinc-400">Write your solution in any supported language and submit to run tests</p>
                                </div>
                                <div className="flex items-start gap-3 text-left">
                                    <span className="text-zinc-500 font-mono">4.</span>
                                    <p className="text-zinc-400">First to pass all tests wins. Compete in ranked mode to climb the leaderboard</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <Footer />
            </div>
        );
    }

    if (mode === 'create') {
        return (
            <div className="min-h-screen bg-zinc-900 flex flex-col">
                <NavBar />
                <div className="flex-1 flex items-center justify-center p-4">
                    <div className="w-full max-w-sm">
                        <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-5">
                            <h2 className="text-lg font-semibold text-white mb-4">Create Room</h2>

                            {/* Game Mode */}
                            <div className="mb-4">
                                <label className="block text-xs font-medium text-zinc-400 mb-2 uppercase tracking-wide">
                                    Mode
                                </label>
                                <div className="grid grid-cols-2 gap-2">
                                    <button
                                        onClick={() => setGameMode('casual')}
                                        className={`py-2 rounded text-sm font-medium transition-colors ${
                                            gameMode === 'casual'
                                                ? 'bg-blue-600 text-white'
                                                : 'bg-zinc-700 text-zinc-300 hover:bg-zinc-600'
                                        }`}
                                    >
                                        Casual
                                    </button>
                                    <button
                                        onClick={() => isAuthenticated && setGameMode('ranked')}
                                        className={`py-2 rounded text-sm font-medium transition-colors ${
                                            gameMode === 'ranked'
                                                ? 'bg-amber-600 text-white'
                                                : 'bg-zinc-700 text-zinc-300 hover:bg-zinc-600'
                                        } ${!isAuthenticated ? 'opacity-50 cursor-not-allowed' : ''}`}
                                    >
                                        Ranked
                                    </button>
                                </div>
                                {!isAuthenticated && (
                                    <p className="text-zinc-500 text-xs mt-1">Sign in to play ranked</p>
                                )}
                            </div>

                            {/* Difficulty */}
                            <div className="mb-4">
                                <label className="block text-xs font-medium text-zinc-400 mb-2 uppercase tracking-wide">
                                    Difficulty
                                </label>
                                <div className="grid grid-cols-4 gap-1.5">
                                    {(['random', 'easy', 'medium', 'hard'] as Difficulty[]).map((d) => (
                                        <button
                                            key={d}
                                            onClick={() => setDifficulty(d)}
                                            className={`py-1.5 rounded text-xs font-medium transition-colors ${
                                                difficulty === d
                                                    ? d === 'easy' ? 'bg-green-600 text-white'
                                                    : d === 'medium' ? 'bg-yellow-600 text-white'
                                                    : d === 'hard' ? 'bg-red-600 text-white'
                                                    : 'bg-purple-600 text-white'
                                                    : 'bg-zinc-700 text-zinc-300 hover:bg-zinc-600'
                                            }`}
                                        >
                                            {d.charAt(0).toUpperCase() + d.slice(1)}
                                        </button>
                                    ))}
                                </div>
                            </div>

                            {/* Player Count */}
                            <div className="mb-5">
                                <label className="block text-xs font-medium text-zinc-400 mb-2 uppercase tracking-wide">
                                    Players
                                </label>
                                <div className="flex items-center gap-3">
                                    <div className="w-9 h-9">
                                        {playerCount > 1 && (
                                            <button
                                                onClick={() => setPlayerCount(playerCount - 1)}
                                                className="w-9 h-9 rounded bg-zinc-700 text-white font-medium hover:bg-zinc-600 transition-colors"
                                            >
                                                -
                                            </button>
                                        )}
                                    </div>
                                    <span className="flex-1 text-center text-xl font-semibold text-white">
                                        {playerCount}
                                    </span>
                                    <div className="w-9 h-9">
                                        {playerCount < 4 && (
                                            <button
                                                onClick={() => setPlayerCount(playerCount + 1)}
                                                className="w-9 h-9 rounded bg-zinc-700 text-white font-medium hover:bg-zinc-600 transition-colors"
                                            >
                                                +
                                            </button>
                                        )}
                                    </div>
                                </div>
                            </div>

                            {/* Summary */}
                            <div className="bg-zinc-900 rounded p-3 mb-4 text-sm">
                                <div className="flex justify-between text-zinc-400 mb-1">
                                    <span>Host</span>
                                    <span className="text-white">{displayName}</span>
                                </div>
                                <div className="flex justify-between text-zinc-400 mb-1">
                                    <span>Mode</span>
                                    <span className={gameMode === 'ranked' ? 'text-amber-400' : 'text-blue-400'}>
                                        {gameMode.charAt(0).toUpperCase() + gameMode.slice(1)}
                                    </span>
                                </div>
                                <div className="flex justify-between text-zinc-400">
                                    <span>Difficulty</span>
                                    <span className={
                                        difficulty === 'easy' ? 'text-green-400' :
                                        difficulty === 'medium' ? 'text-yellow-400' :
                                        difficulty === 'hard' ? 'text-red-400' : 'text-purple-400'
                                    }>
                                        {difficulty.charAt(0).toUpperCase() + difficulty.slice(1)}
                                    </span>
                                </div>
                            </div>

                            <button
                                onClick={handleCreateRoom}
                                disabled={isCreating}
                                className={`w-full py-2.5 rounded text-sm font-medium transition-colors ${
                                    isCreating
                                        ? 'bg-zinc-700 text-zinc-500 cursor-not-allowed'
                                        : 'bg-green-600 hover:bg-green-500 text-white'
                                }`}
                            >
                                {isCreating ? 'Creating...' : 'Create Room'}
                            </button>

                            <button
                                onClick={() => setMode('home')}
                                className="w-full py-2 mt-2 rounded text-sm text-zinc-400 hover:text-white transition-colors"
                            >
                                Cancel
                            </button>
                        </div>
                    </div>
                </div>
                <Footer />
            </div>
        );
    }

    if (mode === 'join') {
        return (
            <div className="min-h-screen bg-zinc-900 flex flex-col">
                <NavBar />
                <div className="flex-1 flex items-center justify-center p-4">
                    <div className="w-full max-w-sm">
                        <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-5">
                            <h2 className="text-lg font-semibold text-white mb-4">Join Room</h2>

                            <div className="mb-4">
                                <label htmlFor="roomCode" className="block text-xs font-medium text-zinc-400 mb-1.5 uppercase tracking-wide">
                                    Room Code
                                </label>
                                <input
                                    type="text"
                                    id="roomCode"
                                    value={roomCode}
                                    onChange={(e) => setRoomCode(e.target.value.toUpperCase())}
                                    className="w-full px-3 py-2.5 bg-zinc-900 border border-zinc-600 rounded text-white text-sm text-center font-mono tracking-wider focus:outline-none focus:border-zinc-500"
                                    placeholder="SWIFT-CODER-1234"
                                    maxLength={17}
                                />
                            </div>

                            <button
                                onClick={handleJoinRoom}
                                disabled={!roomCode.trim()}
                                className={`w-full py-2.5 rounded text-sm font-medium transition-colors ${
                                    roomCode.trim()
                                        ? 'bg-blue-600 hover:bg-blue-500 text-white'
                                        : 'bg-zinc-700 text-zinc-500 cursor-not-allowed'
                                }`}
                            >
                                Join Room
                            </button>

                            <button
                                onClick={() => setMode('home')}
                                className="w-full py-2 mt-2 rounded text-sm text-zinc-400 hover:text-white transition-colors"
                            >
                                Cancel
                            </button>
                        </div>
                    </div>
                </div>
                <Footer />
            </div>
        );
    }

    return null;
}
