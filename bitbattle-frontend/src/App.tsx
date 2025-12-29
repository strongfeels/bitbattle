import { useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { AuthProvider, useAuth } from './contexts/AuthContext.tsx';
import CollaborativeEditor from './components/CollaborativeEditor.tsx';
import RoomLobby from './components/RoomLobby.tsx';
import Leaderboard from './components/Leaderboard.tsx';
import Profile from './components/Profile.tsx';
import UsernameModal from './components/UsernameModal.tsx';
import Logo from './components/Logo.tsx';

type Difficulty = 'random' | 'easy' | 'medium' | 'hard';
type GameMode = 'casual' | 'ranked';

function GameRoom() {
    const [username, setUsername] = useState<string>('');
    const [roomCode, setRoomCode] = useState<string>('');
    const [isInRoom, setIsInRoom] = useState<boolean>(false);
    const [difficulty, setDifficulty] = useState<Difficulty>('random');
    const [requiredPlayers, setRequiredPlayers] = useState<number>(2);
    const [gameMode, setGameMode] = useState<GameMode>('casual');

    const handleJoinRoom = (code: string, user: string, diff?: Difficulty, playerCount?: number, mode?: GameMode) => {
        setRoomCode(code);
        setUsername(user);
        if (diff) setDifficulty(diff);
        if (playerCount) setRequiredPlayers(playerCount);
        if (mode) setGameMode(mode);
        setIsInRoom(true);
    };

    const handleLeaveRoom = () => {
        setIsInRoom(false);
        setRoomCode('');
    };

    if (!isInRoom) {
        return (
            <RoomLobby
                onJoinRoom={handleJoinRoom}
                username={username}
                setUsername={setUsername}
            />
        );
    }

    return (
        <div className="h-screen flex flex-col bg-zinc-900">
            {/* Room Header */}
            <div className="bg-zinc-800 border-b border-zinc-700 px-3 py-2 flex justify-between items-center">
                <div className="flex items-center gap-3 min-w-0">
                    <Logo size="sm" showText={true} />
                    <div className="w-px h-4 bg-zinc-600" />
                    <span className={`text-xs font-medium ${gameMode === 'ranked' ? 'text-amber-400' : 'text-blue-400'}`}>
                        {gameMode === 'ranked' ? 'Ranked' : 'Casual'}
                    </span>
                    <span className="text-zinc-500 text-xs font-mono">
                        {roomCode}
                    </span>
                </div>
                <button
                    onClick={() => {
                        if (window.confirm('Are you sure you want to leave?')) {
                            handleLeaveRoom();
                        }
                    }}
                    className="text-zinc-400 hover:text-white text-sm transition-colors"
                >
                    Leave
                </button>
            </div>

            {/* Main Editor */}
            <div className="flex-1 overflow-hidden">
                <CollaborativeEditor
                    username={username}
                    roomId={roomCode}
                    difficulty={difficulty}
                    requiredPlayers={requiredPlayers}
                    gameMode={gameMode}
                    onLeaveRoom={handleLeaveRoom}
                />
            </div>
        </div>
    );
}

function AppContent() {
    const { isNewUser } = useAuth();

    return (
        <>
            {isNewUser && <UsernameModal />}
            <Routes>
                <Route path="/" element={<GameRoom />} />
                <Route path="/leaderboard" element={<Leaderboard />} />
                <Route path="/profile" element={<Profile />} />
                <Route path="/profile/:userId" element={<Profile />} />
            </Routes>
        </>
    );
}

function App() {
    return (
        <AuthProvider>
            <BrowserRouter>
                <AppContent />
            </BrowserRouter>
        </AuthProvider>
    );
}

export default App;
