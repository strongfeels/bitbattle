import { useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { AuthProvider, useAuth } from './contexts/AuthContext.tsx';
import { ToastProvider, useToast } from './contexts/ToastContext.tsx';
import { ErrorBoundary } from './components/ErrorBoundary.tsx';
import CollaborativeEditor from './components/CollaborativeEditor.tsx';
import RoomLobby from './components/RoomLobby.tsx';
import Leaderboard from './components/Leaderboard.tsx';
import Profile from './components/Profile.tsx';
import LiveGames from './components/LiveGames.tsx';
import SpectatorView from './components/SpectatorView.tsx';
import UsernameModal from './components/UsernameModal.tsx';
import Logo from './components/Logo.tsx';
import { copyToClipboard } from './utils/clipboard.ts';

type Difficulty = 'random' | 'easy' | 'medium' | 'hard';
type GameMode = 'casual' | 'ranked';

function CopyButton({ text, label }: { text: string; label: string }) {
    const { success, error } = useToast();
    const [copied, setCopied] = useState(false);

    const handleCopy = async () => {
        const ok = await copyToClipboard(text);
        if (ok) {
            setCopied(true);
            success('Room code copied to clipboard!');
            setTimeout(() => setCopied(false), 2000);
        } else {
            error('Failed to copy to clipboard');
        }
    };

    return (
        <button
            onClick={handleCopy}
            className="flex items-center gap-1.5 text-zinc-500 hover:text-zinc-300 text-xs font-mono transition-colors group"
            aria-label={`Copy ${label} to clipboard`}
            title="Click to copy"
        >
            <span>{text}</span>
            {copied ? (
                <svg className="w-3.5 h-3.5 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
            ) : (
                <svg className="w-3.5 h-3.5 opacity-0 group-hover:opacity-100 transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
            )}
        </button>
    );
}

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
                    <div className="w-px h-4 bg-zinc-600" aria-hidden="true" />
                    <span
                        className={`text-xs font-medium ${gameMode === 'ranked' ? 'text-amber-400' : 'text-blue-400'}`}
                        aria-label={`Game mode: ${gameMode}`}
                    >
                        {gameMode === 'ranked' ? 'Ranked' : 'Casual'}
                    </span>
                    <CopyButton text={roomCode} label="room code" />
                </div>
                <button
                    onClick={() => {
                        if (window.confirm('Are you sure you want to leave?')) {
                            handleLeaveRoom();
                        }
                    }}
                    className="text-zinc-400 hover:text-white text-sm transition-colors px-2 py-1 rounded hover:bg-zinc-700"
                    aria-label="Leave the current room"
                >
                    Leave
                </button>
            </div>

            {/* Main Editor */}
            <div className="flex-1 overflow-hidden">
                <ErrorBoundary>
                    <CollaborativeEditor
                        username={username}
                        roomId={roomCode}
                        difficulty={difficulty}
                        requiredPlayers={requiredPlayers}
                        gameMode={gameMode}
                        onLeaveRoom={handleLeaveRoom}
                    />
                </ErrorBoundary>
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
                <Route path="/live" element={<LiveGames />} />
                <Route path="/spectate/:roomCode" element={<SpectatorView />} />
            </Routes>
        </>
    );
}

function App() {
    return (
        <ErrorBoundary>
            <AuthProvider>
                <ToastProvider>
                    <BrowserRouter>
                        <AppContent />
                    </BrowserRouter>
                </ToastProvider>
            </AuthProvider>
        </ErrorBoundary>
    );
}

export default App;
