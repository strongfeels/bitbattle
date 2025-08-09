import { useState, useCallback, useRef } from 'react';
import CodeMirrorEditor from './CodeMirrorEditor.tsx';
import { useWebSocket } from '../hooks/useWebSocket.ts';
interface Props {
    roomId?: string;
    username: string;
    initialCode?: string;
    readOnly?: boolean;
}

interface WebSocketMessageData {
    code?: string;
    position?: number;
    username?: string;
    timestamp?: number;
}

interface WebSocketMessage {
    type: 'code_change' | 'cursor_position' | 'user_joined' | 'user_left';
    data: WebSocketMessageData;
}

export default function CollaborativeEditor({
                                                roomId = 'default',
                                                username,
                                                initialCode = '// Welcome to BitBattle!\n// Start coding together!\n\nconsole.log("Hello, world!");',
                                                readOnly = false,
                                            }: Props) {
    const [code, setCode] = useState(initialCode);
    const [connectedUsers, setConnectedUsers] = useState<string[]>([]);
    const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected'>('connecting');

    // Prevent sending changes we received from WebSocket
    const isReceivingUpdate = useRef(false);

    const handleMessage = useCallback((message: string) => {
        try {
            const parsed: WebSocketMessage = JSON.parse(message);

            switch (parsed.type) {
                case 'code_change':
                    if (parsed.data.code !== undefined && typeof parsed.data.code === 'string') {
                        isReceivingUpdate.current = true;
                        setCode(parsed.data.code);
                        // Reset flag after state update
                        setTimeout(() => {
                            isReceivingUpdate.current = false;
                        }, 0);
                    }
                    break;

                case 'user_joined':
                    if (parsed.data.username && typeof parsed.data.username === 'string') {
                        const username = parsed.data.username;
                        setConnectedUsers(prev => [...prev.filter(u => u !== username), username]);
                    }
                    break;

                case 'user_left':
                    if (parsed.data.username && typeof parsed.data.username === 'string') {
                        const username = parsed.data.username;
                        setConnectedUsers(prev => prev.filter(u => u !== username));
                    }
                    break;
            }
        } catch (error) {
            console.error('Failed to parse WebSocket message:', error);
        }
    }, []);

    const { isConnected, sendMessage } = useWebSocket({
        url: `ws://localhost:4000/ws`,
        onMessage: handleMessage,
        onOpen: () => {
            setConnectionStatus('connected');
            // Send user joined message
            const joinMessage: WebSocketMessage = {
                type: 'user_joined',
                data: { username, timestamp: Date.now() }
            };
            sendMessage(JSON.stringify(joinMessage));
        },
        onClose: () => setConnectionStatus('disconnected'),
        shouldReconnect: true,
    });

    const handleCodeChange = useCallback((newCode: string) => {
        // Don't send changes if we're currently receiving an update
        if (isReceivingUpdate.current) return;

        setCode(newCode);

        if (isConnected) {
            const message: WebSocketMessage = {
                type: 'code_change',
                data: {
                    code: newCode,
                    username,
                    timestamp: Date.now(),
                },
            };
            sendMessage(JSON.stringify(message));
        }
    }, [isConnected, sendMessage, username]);

    const getStatusColor = () => {
        switch (connectionStatus) {
            case 'connected': return 'text-green-500';
            case 'connecting': return 'text-yellow-500';
            case 'disconnected': return 'text-red-500';
        }
    };

    const getStatusText = () => {
        switch (connectionStatus) {
            case 'connected': return 'Connected';
            case 'connecting': return 'Connecting...';
            case 'disconnected': return 'Disconnected';
        }
    };

    return (
        <div className="flex flex-col h-full">
            {/* Header with connection status and user info */}
            <div className="flex justify-between items-center p-4 bg-gray-800 text-white">
                <div className="flex items-center space-x-4">
                    <h2 className="text-xl font-bold">BitBattle Editor</h2>
                    <span className="text-sm text-gray-300">Room: {roomId}</span>
                </div>

                <div className="flex items-center space-x-4">
                    <div className="flex items-center space-x-2">
                        <span className="text-sm">Connected Users:</span>
                        <div className="flex space-x-1">
                            {connectedUsers.length === 0 ? (
                                <span className="text-gray-400 text-sm">None</span>
                            ) : (
                                connectedUsers.map(user => (
                                    <span key={user} className="bg-blue-600 px-2 py-1 rounded-full text-xs">
                    {user}
                  </span>
                                ))
                            )}
                        </div>
                    </div>

                    <div className="flex items-center space-x-2">
                        <div className={`w-3 h-3 rounded-full ${connectionStatus === 'connected' ? 'bg-green-500' : connectionStatus === 'connecting' ? 'bg-yellow-500' : 'bg-red-500'}`}></div>
                        <span className={`text-sm ${getStatusColor()}`}>
              {getStatusText()}
            </span>
                    </div>
                </div>
            </div>

            {/* Code Editor */}
            <div className="flex-1">
                <CodeMirrorEditor
                    value={code}
                    onChange={handleCodeChange}
                    readOnly={readOnly}
                    style={{ height: '100%', border: 'none' }}
                />
            </div>

            {/* Footer with user info */}
            <div className="p-2 bg-gray-100 text-sm text-gray-600 flex justify-between">
                <span>Logged in as: <strong>{username}</strong></span>
                <span>Language: JavaScript</span>
            </div>
        </div>
    );
}