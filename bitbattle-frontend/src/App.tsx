import { useState } from 'react';
import CollaborativeEditor from './components/CollaborativeEditor.tsx';
import RoomLobby from './components/RoomLobby.tsx';

function App() {
    const [username, setUsername] = useState<string>('');
    const [roomCode, setRoomCode] = useState<string>('');
    const [isInRoom, setIsInRoom] = useState<boolean>(false);

    const handleJoinRoom = (code: string, user: string) => {
        setRoomCode(code);
        setUsername(user);
        setIsInRoom(true);
    };

    const handleLeaveRoom = () => {
        setIsInRoom(false);
        setRoomCode('');
        // Keep username for easy re-joining
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
        <div className="h-screen flex flex-col">
            {/* Room Header */}
            <div className="bg-gray-900 text-white px-4 py-2 flex justify-between items-center">
                <div className="flex items-center space-x-4">
                    <h1 className="text-lg font-bold">⚔️ BitBattle</h1>
                    <div className="bg-blue-600 px-3 py-1 rounded-full text-sm font-mono">
                        {roomCode}
                    </div>
                </div>
                <button
                    onClick={handleLeaveRoom}
                    className="bg-red-600 hover:bg-red-700 px-3 py-1 rounded text-sm transition-colors"
                >
                    Leave Room
                </button>
            </div>

            {/* Main Editor */}
            <div className="flex-1">
                <CollaborativeEditor
                    username={username}
                    roomId={roomCode}
                />
            </div>
        </div>
    );
}

export default App;