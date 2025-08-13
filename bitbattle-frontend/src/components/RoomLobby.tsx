import { useState } from 'react';
import { generateRoomCode, isValidRoomCode, formatRoomCode } from '../utils/roomUtils.ts';

interface Props {
    onJoinRoom: (roomCode: string, username: string) => void;
    username: string;
    setUsername: (username: string) => void;
}

export default function RoomLobby({ onJoinRoom, username, setUsername }: Props) {
    const [mode, setMode] = useState<'home' | 'create' | 'join'>('home');
    const [roomCode, setRoomCode] = useState('');
    const [isCreating, setIsCreating] = useState(false);

    const handleCreateRoom = async () => {
        if (!username.trim()) return;

        setIsCreating(true);
        const newRoomCode = generateRoomCode();

        // Simulate room creation delay
        setTimeout(() => {
            setIsCreating(false);
            onJoinRoom(newRoomCode, username.trim());
        }, 1000);
    };

    const handleJoinRoom = () => {
        if (!username.trim() || !roomCode.trim()) return;

        const formattedCode = formatRoomCode(roomCode.trim());
        if (!isValidRoomCode(formattedCode)) {
            alert('Please enter a valid room code (e.g., SWIFT-CODER-1234)');
            return;
        }

        onJoinRoom(formattedCode, username.trim());
    };

    const handleQuickJoin = () => {
        if (!username.trim()) return;
        // Join a "default" room for quick access
        onJoinRoom('QUICK-BATTLE-0001', username.trim());
    };

    if (mode === 'home') {
        return (
            <div className="min-h-screen bg-gradient-to-br from-blue-900 via-purple-900 to-indigo-900 flex items-center justify-center p-4">
                <div className="max-w-md w-full space-y-8">
                    {/* Header */}
                    <div className="text-center">
                        <h1 className="text-4xl font-bold text-white mb-2">⚔️ BitBattle</h1>
                        <p className="text-blue-200">Real-time collaborative coding challenges</p>
                    </div>

                    {/* Username Input */}
                    <div className="bg-white/10 backdrop-blur-md rounded-2xl p-6 space-y-4">
                        <div>
                            <label htmlFor="username" className="block text-sm font-medium text-white mb-2">
                                Choose your battle name
                            </label>
                            <input
                                type="text"
                                id="username"
                                value={username}
                                onChange={(e) => setUsername(e.target.value)}
                                className="w-full px-4 py-3 bg-white/20 border border-white/30 rounded-lg text-white placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent"
                                placeholder="Enter your username..."
                                maxLength={20}
                                required
                            />
                        </div>

                        {/* Action Buttons */}
                        <div className="space-y-3">
                            <button
                                onClick={() => setMode('create')}
                                disabled={!username.trim()}
                                className={`w-full py-3 px-4 rounded-lg font-medium transition-all ${
                                    username.trim()
                                        ? 'bg-green-600 hover:bg-green-700 text-white shadow-lg hover:shadow-xl'
                                        : 'bg-gray-600 text-gray-300 cursor-not-allowed'
                                }`}
                            >
                                🎯 Create New Battle Room
                            </button>

                            <button
                                onClick={() => setMode('join')}
                                disabled={!username.trim()}
                                className={`w-full py-3 px-4 rounded-lg font-medium transition-all ${
                                    username.trim()
                                        ? 'bg-blue-600 hover:bg-blue-700 text-white shadow-lg hover:shadow-xl'
                                        : 'bg-gray-600 text-gray-300 cursor-not-allowed'
                                }`}
                            >
                                🚪 Join Existing Room
                            </button>

                            <button
                                onClick={handleQuickJoin}
                                disabled={!username.trim()}
                                className={`w-full py-3 px-4 rounded-lg font-medium transition-all border-2 ${
                                    username.trim()
                                        ? 'border-purple-400 text-purple-200 hover:bg-purple-400 hover:text-white'
                                        : 'border-gray-600 text-gray-300 cursor-not-allowed'
                                }`}
                            >
                                ⚡ Quick Battle (Public Room)
                            </button>
                        </div>
                    </div>

                    {/* Footer */}
                    <div className="text-center text-blue-300 text-sm">
                        <p>Solve coding challenges together in real-time</p>
                    </div>
                </div>
            </div>
        );
    }

    if (mode === 'create') {
        return (
            <div className="min-h-screen bg-gradient-to-br from-green-900 via-emerald-900 to-teal-900 flex items-center justify-center p-4">
                <div className="max-w-md w-full">
                    <div className="bg-white/10 backdrop-blur-md rounded-2xl p-6 space-y-6">
                        <div className="text-center">
                            <h2 className="text-2xl font-bold text-white mb-2">🎯 Create Battle Room</h2>
                            <p className="text-green-200">You'll be the room host</p>
                        </div>

                        <div className="space-y-4">
                            <div className="bg-white/20 rounded-lg p-4">
                                <h3 className="text-white font-medium mb-2">Room Details:</h3>
                                <ul className="text-green-200 text-sm space-y-1">
                                    <li>• Host: <strong>{username}</strong></li>
                                    <li>• Max players: 8</li>
                                    <li>• Random coding challenge assigned</li>
                                    <li>• Real-time collaboration</li>
                                </ul>
                            </div>

                            <button
                                onClick={handleCreateRoom}
                                disabled={isCreating}
                                className={`w-full py-3 px-4 rounded-lg font-medium transition-all ${
                                    isCreating
                                        ? 'bg-gray-600 text-gray-300 cursor-not-allowed'
                                        : 'bg-green-600 hover:bg-green-700 text-white shadow-lg hover:shadow-xl'
                                }`}
                            >
                                {isCreating ? '🔄 Creating Room...' : '🚀 Create & Enter Room'}
                            </button>

                            <button
                                onClick={() => setMode('home')}
                                className="w-full py-2 px-4 rounded-lg font-medium text-green-200 hover:bg-white/10 transition-all"
                            >
                                ← Back to Main Menu
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    if (mode === 'join') {
        return (
            <div className="min-h-screen bg-gradient-to-br from-blue-900 via-indigo-900 to-purple-900 flex items-center justify-center p-4">
                <div className="max-w-md w-full">
                    <div className="bg-white/10 backdrop-blur-md rounded-2xl p-6 space-y-6">
                        <div className="text-center">
                            <h2 className="text-2xl font-bold text-white mb-2">🚪 Join Battle Room</h2>
                            <p className="text-blue-200">Enter the room code shared by your host</p>
                        </div>

                        <div className="space-y-4">
                            <div>
                                <label htmlFor="roomCode" className="block text-sm font-medium text-white mb-2">
                                    Room Code
                                </label>
                                <input
                                    type="text"
                                    id="roomCode"
                                    value={roomCode}
                                    onChange={(e) => setRoomCode(e.target.value.toUpperCase())}
                                    className="w-full px-4 py-3 bg-white/20 border border-white/30 rounded-lg text-white placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent text-center text-lg font-mono tracking-wider"
                                    placeholder="SWIFT-CODER-1234"
                                    maxLength={17}
                                    required
                                />
                                <p className="text-blue-300 text-xs mt-1">
                                    Format: WORD-WORD-1234
                                </p>
                            </div>

                            <button
                                onClick={handleJoinRoom}
                                disabled={!username.trim() || !roomCode.trim()}
                                className={`w-full py-3 px-4 rounded-lg font-medium transition-all ${
                                    username.trim() && roomCode.trim()
                                        ? 'bg-blue-600 hover:bg-blue-700 text-white shadow-lg hover:shadow-xl'
                                        : 'bg-gray-600 text-gray-300 cursor-not-allowed'
                                }`}
                            >
                                🎮 Join Battle Room
                            </button>

                            <button
                                onClick={() => setMode('home')}
                                className="w-full py-2 px-4 rounded-lg font-medium text-blue-200 hover:bg-white/10 transition-all"
                            >
                                ← Back to Main Menu
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    return null;
}