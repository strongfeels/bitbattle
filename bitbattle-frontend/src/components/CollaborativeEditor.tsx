import { useState, useCallback, useRef, useEffect } from 'react';
import CodeMirrorEditor from './CodeMirrorEditor.tsx';
import ProblemPanel from './ProblemPanel.tsx';
import { useWebSocket } from '../hooks/useWebSocket.ts';

type Difficulty = 'random' | 'easy' | 'medium' | 'hard';
type GameState = 'waiting' | 'countdown' | 'playing' | 'room_full';
type GameMode = 'casual' | 'ranked';

interface Props {
    roomId?: string;
    username: string;
    onLeaveRoom?: () => void;
    initialCode?: string;
    readOnly?: boolean;
    difficulty?: Difficulty;
    requiredPlayers?: number;
    gameMode?: GameMode;
}

interface TestCase {
    input: string;
    expected_output: string;
    explanation?: string;
}

interface Problem {
    id: string;
    title: string;
    description: string;
    difficulty: 'Easy' | 'Medium' | 'Hard';
    examples: TestCase[];
    starter_code: Record<string, string>;
    time_limit_minutes?: number;
    tags: string[];
}

interface TestResult {
    input: string;
    expected_output: string;
    actual_output: string;
    passed: boolean;
    execution_time_ms: number;
    error?: string;
}

interface SubmissionResult {
    username: string;
    problem_id: string;
    passed: boolean;
    total_tests: number;
    passed_tests: number;
    test_results: TestResult[];
    execution_time_ms: number;
    submission_time: number;
}

interface WebSocketMessageData {
    code?: string;
    position?: number;
    username?: string;
    timestamp?: number;
    problem?: Problem;
    result?: SubmissionResult;
    current?: number;
    required?: number;
    message?: string;
}

interface WebSocketMessage {
    type: 'code_change' | 'cursor_position' | 'user_joined' | 'user_left' | 'problem_assigned' | 'submission_result' | 'player_count' | 'game_start' | 'room_full';
    data: WebSocketMessageData;
}

interface UserEditor {
    username: string;
    code: string;
    lastUpdate: number;
}

type MobileTab = 'problem' | 'code' | 'results';

// Helper function to get default starter code for each language
function getDefaultStarterCode(language: string): string {
    switch (language) {
        case 'python':
            return '# Welcome to BitBattle!\n# Loading problem...\n\ndef solution():\n    # Your solution here\n    pass\n\nprint(solution())';
        case 'java':
            return '// Welcome to BitBattle!\n// Loading problem...\n\nclass Solution {\n    public static void main(String[] args) {\n        // Your solution here\n        System.out.println("Hello, world!");\n    }\n}';
        case 'c':
            return '// Welcome to BitBattle!\n// Loading problem...\n\n#include <stdio.h>\n\nint main() {\n    // Your solution here\n    printf("Hello, world!\\n");\n    return 0;\n}';
        case 'cpp':
            return '// Welcome to BitBattle!\n// Loading problem...\n\n#include <iostream>\nusing namespace std;\n\nint main() {\n    // Your solution here\n    cout << "Hello, world!" << endl;\n    return 0;\n}';
        case 'rust':
            return '// Welcome to BitBattle!\n// Loading problem...\n\nfn main() {\n    // Your solution here\n    println!("Hello, world!");\n}';
        case 'go':
            return '// Welcome to BitBattle!\n// Loading problem...\n\npackage main\n\nimport "fmt"\n\nfunc main() {\n    // Your solution here\n    fmt.Println("Hello, world!")\n}';
        case 'javascript':
        default:
            return '// Welcome to BitBattle!\n// Loading problem...\n\nfunction solution() {\n    // Your solution here\n    return "Hello, world!";\n}\n\nconsole.log(solution());';
    }
}

function TestResultsDisplay({ result }: { result: SubmissionResult }) {
    return (
        <div className="space-y-2">
            {/* Overall Result */}
            <div className={`p-2 rounded-lg ${
                result.passed
                    ? 'bg-green-100 border border-green-300'
                    : 'bg-red-100 border border-red-300'
            }`}>
                <div className="flex items-center space-x-1">
                    <span className="text-sm">
                        {result.passed ? '✅' : '❌'}
                    </span>
                    <span className={`font-semibold text-xs ${
                        result.passed ? 'text-green-800' : 'text-red-800'
                    }`}>
                        {result.passed ? 'Passed!' : 'Failed'}
                    </span>
                </div>
                <div className="text-xs text-gray-600">
                    {result.passed_tests}/{result.total_tests} tests
                </div>
                <div className="text-xs text-gray-500">
                    {result.execution_time_ms}ms
                </div>
            </div>

            {/* Individual Test Results - Condensed */}
            <div className="space-y-1">
                <h4 className="font-medium text-gray-700 text-xs">Tests:</h4>
                {result.test_results.map((test, index) => (
                    <div key={index} className={`p-1 rounded text-xs ${
                        test.passed
                            ? 'bg-green-50 text-green-700'
                            : 'bg-red-50 text-red-700'
                    }`}>
                        <div className="flex items-center space-x-1">
                            <span>{test.passed ? '✅' : '❌'}</span>
                            <span>Test {index + 1}</span>
                        </div>

                        {!test.passed && (
                            <div className="mt-1">
                                <div className="truncate">
                                    <span className="font-medium">Expected: </span>
                                    <code className="text-xs">{test.expected_output}</code>
                                </div>
                                <div className="truncate">
                                    <span className="font-medium">Got: </span>
                                    <code className="text-xs">{test.actual_output || 'No output'}</code>
                                </div>
                                {test.error && (
                                    <div className="text-red-600 text-xs truncate">
                                        Error: {test.error}
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
}

export default function CollaborativeEditor({
                                                roomId = 'default',
                                                username,
                                                onLeaveRoom,
                                                initialCode = '// Welcome to BitBattle!\n// Loading problem...',
                                                readOnly = false,
                                                difficulty = 'random',
                                                requiredPlayers = 1,
                                                gameMode = 'casual',
                                            }: Props) {
    const [userEditors, setUserEditors] = useState<Map<string, UserEditor>>(new Map());
    const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected'>('connecting');
    const [currentProblem, setCurrentProblem] = useState<Problem | null>(null);
    const [timeRemaining, setTimeRemaining] = useState<number | undefined>(undefined);
    const [submissionResults, setSubmissionResults] = useState<Map<string, SubmissionResult>>(new Map());
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [selectedLanguage, setSelectedLanguage] = useState('javascript');
    const [mobileTab, setMobileTab] = useState<MobileTab>('code');
    const [gameState, setGameState] = useState<GameState>('waiting');
    const [countdown, setCountdown] = useState<number>(3);
    const [connectedPlayerCount, setConnectedPlayerCount] = useState<number>(0);
    const timerRef = useRef<NodeJS.Timeout>(undefined);

    // Initialize current user's editor
    useEffect(() => {
        const code = currentProblem?.starter_code?.[selectedLanguage] ||
            currentProblem?.starter_code?.javascript ||
            getDefaultStarterCode(selectedLanguage);
        setUserEditors(prev => {
            const newMap = new Map(prev);
            if (!newMap.has(username)) {
                newMap.set(username, {
                    username,
                    code,
                    lastUpdate: Date.now()
                });
            }
            return newMap;
        });
    }, [username, initialCode, currentProblem, selectedLanguage]);

    // Timer for time-limited problems
    useEffect(() => {
        if (currentProblem?.time_limit_minutes && timeRemaining === undefined) {
            const totalSeconds = currentProblem.time_limit_minutes * 60;
            setTimeRemaining(totalSeconds);
        }
    }, [currentProblem, timeRemaining]);

    useEffect(() => {
        if (timeRemaining !== undefined && timeRemaining > 0) {
            timerRef.current = setTimeout(() => {
                setTimeRemaining(prev => prev !== undefined ? prev - 1 : undefined);
            }, 1000);
        }

        return () => {
            if (timerRef.current) {
                clearTimeout(timerRef.current);
            }
        };
    }, [timeRemaining]);

    const handleMessage = useCallback((message: string) => {
        console.log('📥 Received WebSocket message:', message);

        try {
            const parsed: WebSocketMessage = JSON.parse(message);
            console.log('✅ Parsed message:', parsed);

            switch (parsed.type) {
                case 'submission_result':
                    console.log('🧪 Submission result received:', parsed.data.result);
                    if (parsed.data.result) {
                        setSubmissionResults(prev => {
                            const newMap = new Map(prev);
                            newMap.set(parsed.data.result!.username, parsed.data.result!);
                            return newMap;
                        });
                        // Auto-switch to results tab on mobile after submission
                        if (parsed.data.result.username === username) {
                            setMobileTab('results');
                        }
                    }
                    break;

                case 'problem_assigned':
                    console.log('🎯 Problem assigned:', parsed.data.problem);
                    if (parsed.data.problem) {
                        setCurrentProblem(parsed.data.problem);
                        // Don't start timer here - wait for game_start
                    }
                    break;

                case 'code_change':
                    console.log('🔄 Processing code_change for user:', parsed.data.username);
                    if (parsed.data.code !== undefined &&
                        typeof parsed.data.code === 'string' &&
                        parsed.data.username &&
                        parsed.data.username !== username) { // Don't apply changes from self

                        setUserEditors(prev => {
                            const newMap = new Map(prev);
                            newMap.set(parsed.data.username!, {
                                username: parsed.data.username!,
                                code: parsed.data.code!,
                                lastUpdate: parsed.data.timestamp || Date.now()
                            });
                            return newMap;
                        });
                    } else if (parsed.data.username === username) {
                        console.log('🔄 Ignoring own code change from server');
                    }
                    break;

                case 'user_joined':
                    console.log('👤 User joined:', parsed.data.username);
                    if (parsed.data.username && typeof parsed.data.username === 'string') {
                        setUserEditors(prev => {
                            const newMap = new Map(prev);
                            // Limit to 4 users maximum
                            if (newMap.size >= 4 && !newMap.has(parsed.data.username!)) {
                                console.log('🚫 Room is full (4 users max)');
                                return prev;
                            }

                            if (!newMap.has(parsed.data.username!)) {
                                const code = currentProblem?.starter_code?.[selectedLanguage] ||
                                    currentProblem?.starter_code?.javascript ||
                                    getDefaultStarterCode(selectedLanguage);
                                newMap.set(parsed.data.username!, {
                                    username: parsed.data.username!,
                                    code,
                                    lastUpdate: Date.now()
                                });
                            }
                            return newMap;
                        });
                    }
                    break;

                case 'user_left':
                    console.log('👋 User left:', parsed.data.username);
                    if (parsed.data.username && typeof parsed.data.username === 'string') {
                        setUserEditors(prev => {
                            const newMap = new Map(prev);
                            newMap.delete(parsed.data.username!);
                            return newMap;
                        });
                    }
                    break;

                case 'player_count':
                    console.log('👥 Player count update:', parsed.data.current, '/', parsed.data.required);
                    setConnectedPlayerCount(parsed.data.current || 0);
                    break;

                case 'game_start':
                    console.log('🎮 Game starting! Beginning countdown...');
                    setGameState('countdown');
                    setCountdown(3);
                    // Start countdown
                    let count = 3;
                    const countdownInterval = setInterval(() => {
                        count--;
                        setCountdown(count);
                        if (count <= 0) {
                            clearInterval(countdownInterval);
                            setGameState('playing');
                            // Now start the timer
                            if (currentProblem?.time_limit_minutes) {
                                setTimeRemaining(currentProblem.time_limit_minutes * 60);
                            }
                        }
                    }, 1000);
                    break;

                case 'room_full':
                    console.log('🚫 Room is full:', parsed.data.message);
                    setGameState('room_full');
                    break;

                default:
                    console.log('❓ Unknown message type:', parsed.type);
                    break;
            }
        } catch (error) {
            console.error('❌ Failed to parse WebSocket message:', error);
        }
    }, [username, initialCode, currentProblem, selectedLanguage]);

    const { sendMessage } = useWebSocket({
        url: `ws://localhost:4000/ws?room=${encodeURIComponent(roomId)}&difficulty=${encodeURIComponent(difficulty)}&players=${requiredPlayers}&mode=${encodeURIComponent(gameMode)}`,
        onMessage: handleMessage,
        onOpen: () => {
            console.log('🟢 CollaborativeEditor: WebSocket connected');
            setConnectionStatus('connected');
            const joinMessage: WebSocketMessage = {
                type: 'user_joined',
                data: { username, timestamp: Date.now() }
            };
            console.log('📤 Sending join message:', joinMessage);
            setTimeout(() => {
                sendMessage(JSON.stringify(joinMessage));
            }, 100);
        },
        onClose: () => {
            console.log('🔴 CollaborativeEditor: WebSocket disconnected');
            setConnectionStatus('disconnected');
        },
        shouldReconnect: true,
    });

    const handleCodeSubmit = useCallback(async () => {
        if (!currentProblem || isSubmitting) return;

        const currentUserEditor = userEditors.get(username);
        if (!currentUserEditor) return;

        setIsSubmitting(true);

        try {
            const response = await fetch('http://localhost:4000/submit', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    username,
                    problem_id: currentProblem.id,
                    code: currentUserEditor.code,
                    language: selectedLanguage,
                    room_id: roomId,
                }),
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const result: SubmissionResult = await response.json();
            console.log('🎯 Submission successful:', result);

            // Update local results immediately
            setSubmissionResults(prev => {
                const newMap = new Map(prev);
                newMap.set(username, result);
                return newMap;
            });

            // Auto-switch to results tab on mobile
            setMobileTab('results');

        } catch (error) {
            console.error('❌ Submission failed:', error);
            // You could add a toast notification here
        } finally {
            setIsSubmitting(false);
        }
    }, [currentProblem, username, userEditors, isSubmitting, selectedLanguage, roomId]);

    const handleLanguageChange = useCallback((newLanguage: string) => {
        setSelectedLanguage(newLanguage);

        // Update current user's code to the starter code for the new language
        if (currentProblem) {
            const newCode = currentProblem.starter_code?.[newLanguage] ||
                getDefaultStarterCode(newLanguage);

            setUserEditors(prev => {
                const newMap = new Map(prev);
                newMap.set(username, {
                    username,
                    code: newCode,
                    lastUpdate: Date.now()
                });
                return newMap;
            });

            // Send the language change to other users
            const message: WebSocketMessage = {
                type: 'code_change',
                data: { code: newCode, username, timestamp: Date.now() }
            };
            sendMessage(JSON.stringify(message));
        }
    }, [currentProblem, username, sendMessage]);

    const handleCodeChange = useCallback((newCode: string) => {
        console.log('🔵 Code changed for current user:', username);

        // Update local state immediately
        setUserEditors(prev => {
            const newMap = new Map(prev);
            newMap.set(username, {
                username,
                code: newCode,
                lastUpdate: Date.now()
            });
            return newMap;
        });

        // Send to other users
        const message: WebSocketMessage = {
            type: 'code_change',
            data: { code: newCode, username, timestamp: Date.now() }
        };
        console.log('📤 Sending code change for user:', username);
        sendMessage(JSON.stringify(message));
    }, [username, sendMessage]);

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

    const userEditorsArray = Array.from(userEditors.values());
    const currentUserEditor = userEditors.get(username);
    const currentUserResult = submissionResults.get(username);

    // Show waiting screen while waiting for players
    if (gameState === 'waiting') {
        return (
            <div className="h-full flex items-center justify-center bg-gradient-to-br from-blue-900 via-purple-900 to-indigo-900">
                <div className="text-center space-y-6 p-8">
                    <div className="text-6xl animate-pulse">⏳</div>
                    <h2 className="text-3xl font-bold text-white">Waiting for Players</h2>
                    <div className="text-xl text-blue-200">
                        <span className="text-4xl font-bold text-green-400">{connectedPlayerCount}</span>
                        <span className="mx-2">/</span>
                        <span className="text-4xl font-bold text-white">{requiredPlayers}</span>
                        <span className="ml-2">players joined</span>
                    </div>
                    <div className="flex justify-center space-x-2">
                        {Array.from({ length: requiredPlayers }).map((_, i) => (
                            <div
                                key={i}
                                className={`w-4 h-4 rounded-full transition-all duration-300 ${
                                    i < connectedPlayerCount
                                        ? 'bg-green-500 scale-110'
                                        : 'bg-gray-600'
                                }`}
                            />
                        ))}
                    </div>
                    <p className="text-gray-400 text-sm">
                        Share the room code with your friends to start the battle!
                    </p>
                    <div className="bg-white/10 rounded-lg px-6 py-3 inline-block">
                        <span className="text-gray-400 text-sm">Room Code:</span>
                        <span className="ml-2 text-xl font-mono text-white">{roomId}</span>
                    </div>
                </div>
            </div>
        );
    }

    // Show room full screen
    if (gameState === 'room_full') {
        return (
            <div className="h-full flex items-center justify-center bg-gradient-to-br from-red-900 via-rose-900 to-pink-900">
                <div className="text-center space-y-6 p-8">
                    <div className="text-6xl">🚫</div>
                    <h2 className="text-3xl font-bold text-white">Room is Full</h2>
                    <p className="text-xl text-red-200">
                        This room has already started or is at capacity.
                    </p>
                    <p className="text-gray-300">
                        Please try joining a different room or create a new one.
                    </p>
                    {onLeaveRoom && (
                        <button
                            onClick={onLeaveRoom}
                            className="mt-4 px-8 py-3 bg-white/20 hover:bg-white/30 text-white font-medium rounded-lg transition-colors"
                        >
                            Return to Lobby
                        </button>
                    )}
                </div>
            </div>
        );
    }

    // Show countdown screen
    if (gameState === 'countdown') {
        return (
            <div className="h-full flex items-center justify-center bg-gradient-to-br from-green-900 via-emerald-900 to-teal-900">
                <div className="text-center space-y-6">
                    <h2 className="text-3xl font-bold text-white">Get Ready!</h2>
                    <div className="text-9xl font-bold text-white animate-bounce">
                        {countdown}
                    </div>
                    <p className="text-xl text-green-200">Battle starts in...</p>
                </div>
            </div>
        );
    }

    return (
        <div className="flex flex-col lg:flex-row h-full">
            {/* Mobile Tab Navigation */}
            <div className="lg:hidden flex border-b border-gray-300 bg-gray-100">
                <button
                    onClick={() => setMobileTab('problem')}
                    className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                        mobileTab === 'problem'
                            ? 'bg-white text-blue-600 border-b-2 border-blue-600'
                            : 'text-gray-600 hover:text-gray-800'
                    }`}
                >
                    Problem
                </button>
                <button
                    onClick={() => setMobileTab('code')}
                    className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                        mobileTab === 'code'
                            ? 'bg-white text-blue-600 border-b-2 border-blue-600'
                            : 'text-gray-600 hover:text-gray-800'
                    }`}
                >
                    Code
                </button>
                <button
                    onClick={() => setMobileTab('results')}
                    className={`flex-1 py-3 px-4 text-sm font-medium transition-colors relative ${
                        mobileTab === 'results'
                            ? 'bg-white text-blue-600 border-b-2 border-blue-600'
                            : 'text-gray-600 hover:text-gray-800'
                    }`}
                >
                    Results
                    {currentUserResult && (
                        <span className={`absolute top-1 right-2 w-2 h-2 rounded-full ${
                            currentUserResult.passed ? 'bg-green-500' : 'bg-red-500'
                        }`} />
                    )}
                </button>
            </div>

            {/* Problem Panel - Left Side (Desktop) / Tab Content (Mobile) */}
            <div className={`${mobileTab === 'problem' ? 'block' : 'hidden'} lg:block w-full lg:w-1/3 lg:border-r border-gray-300 overflow-auto`}>
                <ProblemPanel
                    problem={currentProblem}
                    timeRemaining={timeRemaining}
                />
            </div>

            {/* Coding Arena - Right Side (Desktop) / Tab Content (Mobile) */}
            <div className={`${mobileTab === 'code' ? 'flex' : 'hidden'} lg:flex flex-1 flex-col`}>
                {/* Header with connection status and user info */}
                <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center p-2 sm:p-4 bg-gray-800 text-white gap-2">
                    <div className="flex items-center space-x-2 sm:space-x-4">
                        <h2 className="text-base sm:text-xl font-bold">BitBattle Arena</h2>
                        <span className="text-xs sm:text-sm text-gray-300 hidden sm:inline">Room: {roomId}</span>
                    </div>

                    <div className="flex items-center space-x-2 sm:space-x-4 w-full sm:w-auto justify-between sm:justify-end">
                        <div className="flex items-center space-x-1 sm:space-x-2">
                            <span className="text-xs sm:text-sm hidden sm:inline">Connected Users:</span>
                            <div className="flex space-x-1">
                                {userEditorsArray.length === 0 ? (
                                    <span className="text-gray-400 text-xs sm:text-sm">None</span>
                                ) : (
                                    userEditorsArray.map(editor => (
                                        <span
                                            key={editor.username}
                                            className={`px-1.5 sm:px-2 py-0.5 sm:py-1 rounded-full text-xs ${
                                                editor.username === username
                                                    ? 'bg-green-600'
                                                    : 'bg-blue-600'
                                            }`}
                                        >
                                            {editor.username} {editor.username === username ? '(You)' : ''}
                                        </span>
                                    ))
                                )}
                            </div>
                        </div>

                        <div className="flex items-center space-x-1 sm:space-x-2">
                            <div className={`w-2 h-2 sm:w-3 sm:h-3 rounded-full ${connectionStatus === 'connected' ? 'bg-green-500' : connectionStatus === 'connecting' ? 'bg-yellow-500' : 'bg-red-500'}`}></div>
                            <span className={`text-xs sm:text-sm ${getStatusColor()}`}>
                                {getStatusText()}
                            </span>
                        </div>
                    </div>
                </div>

                {/* Editors Grid - Desktop shows all users, Mobile shows only current user */}
                <div className="flex-1 flex">
                    {userEditorsArray.length === 0 ? (
                        <div className="flex-1 flex items-center justify-center bg-gray-100">
                            <span className="text-gray-500 text-lg">Waiting for users to join...</span>
                        </div>
                    ) : (
                        <>
                            {/* Mobile: Only show current user's editor */}
                            <div className="lg:hidden flex-1 flex flex-col">
                                {currentUserEditor && (
                                    <>
                                        {/* Editor Header with Language Selector */}
                                        <div className="p-2 bg-green-100 text-green-800 flex items-center justify-between">
                                            <span className="text-sm font-medium">Your Editor</span>
                                            <select
                                                value={selectedLanguage}
                                                onChange={(e) => handleLanguageChange(e.target.value)}
                                                className="px-2 py-1 text-xs border border-green-300 rounded bg-white text-gray-800 focus:outline-none focus:ring-1 focus:ring-green-500"
                                            >
                                                <option value="c">C</option>
                                                <option value="cpp">C++</option>
                                                <option value="go">Go</option>
                                                <option value="java">Java</option>
                                                <option value="javascript">JavaScript</option>
                                                <option value="python">Python</option>
                                                <option value="rust">Rust</option>
                                            </select>
                                        </div>

                                        {/* Code Editor */}
                                        <div className="flex-1">
                                            <CodeMirrorEditor
                                                value={currentUserEditor.code}
                                                onChange={handleCodeChange}
                                                readOnly={readOnly}
                                                language={selectedLanguage}
                                                style={{ height: '100%', border: 'none' }}
                                            />
                                        </div>

                                        {/* Submit Button */}
                                        {currentProblem && (
                                            <div className="p-3 bg-gray-50 border-t">
                                                <button
                                                    onClick={handleCodeSubmit}
                                                    disabled={isSubmitting}
                                                    className={`w-full py-3 px-4 rounded-md font-medium transition-colors text-sm ${
                                                        isSubmitting
                                                            ? 'bg-gray-400 text-gray-700 cursor-not-allowed'
                                                            : 'bg-green-600 hover:bg-green-700 text-white'
                                                    }`}
                                                >
                                                    {isSubmitting ? 'Running...' : 'Submit Solution'}
                                                </button>
                                            </div>
                                        )}
                                    </>
                                )}
                            </div>

                            {/* Desktop: Show all users' editors */}
                            <div className="hidden lg:flex flex-1">
                                {userEditorsArray.map((editor, index) => {
                                    // Calculate width based on number of users (max 4)
                                    const userCount = Math.min(userEditorsArray.length, 4);
                                    let widthClass = '';

                                    if (userCount === 1) {
                                        widthClass = 'w-full';
                                    } else if (userCount === 2) {
                                        widthClass = 'w-1/2';
                                    } else if (userCount === 3) {
                                        widthClass = 'w-1/3';
                                    } else {
                                        widthClass = 'w-1/4'; // 4 users max
                                    }

                                    return (
                                        <div
                                            key={editor.username}
                                            className={`${widthClass} flex ${index > 0 ? 'border-l border-gray-300' : ''}`}
                                        >
                                            {/* Editor Section */}
                                            <div className="flex-1 flex flex-col">
                                                {/* Editor Header */}
                                                <div className={`p-2 text-sm font-medium flex items-center justify-between ${
                                                    editor.username === username
                                                        ? 'bg-green-100 text-green-800'
                                                        : 'bg-blue-100 text-blue-800'
                                                }`}>
                                                    <span>{editor.username} {editor.username === username ? '(You)' : ''}</span>
                                                    {editor.username === username && (
                                                        <select
                                                            value={selectedLanguage}
                                                            onChange={(e) => handleLanguageChange(e.target.value)}
                                                            className="px-2 py-1 text-xs border border-green-300 rounded bg-white text-gray-800 focus:outline-none focus:ring-1 focus:ring-green-500"
                                                        >
                                                            <option value="c">C</option>
                                                            <option value="cpp">C++</option>
                                                            <option value="go">Go</option>
                                                            <option value="java">Java</option>
                                                            <option value="javascript">JavaScript</option>
                                                            <option value="python">Python</option>
                                                            <option value="rust">Rust</option>
                                                        </select>
                                                    )}
                                                </div>

                                                {/* Code Editor */}
                                                <div className="flex-1">
                                                    <CodeMirrorEditor
                                                        value={editor.code}
                                                        onChange={editor.username === username ? handleCodeChange : undefined}
                                                        readOnly={editor.username !== username || readOnly}
                                                        language={selectedLanguage}
                                                        style={{ height: '100%', border: 'none' }}
                                                    />
                                                </div>

                                                {/* Submit Button - Only for current user */}
                                                {editor.username === username && currentProblem && (
                                                    <div className="p-3 bg-gray-50 border-t">
                                                        <button
                                                            onClick={handleCodeSubmit}
                                                            disabled={isSubmitting}
                                                            className={`w-full py-2 px-4 rounded-md font-medium transition-colors text-sm ${
                                                                isSubmitting
                                                                    ? 'bg-gray-400 text-gray-700 cursor-not-allowed'
                                                                    : 'bg-green-600 hover:bg-green-700 text-white'
                                                            }`}
                                                        >
                                                            {isSubmitting ? 'Running...' : 'Submit'}
                                                        </button>
                                                    </div>
                                                )}
                                            </div>

                                            {/* Test Results Panel - Only show for users with results */}
                                            {submissionResults.has(editor.username) && (
                                                <div className={`border-l border-gray-300 bg-gray-50 overflow-y-auto ${
                                                    userCount === 1 ? 'w-80' : userCount === 2 ? 'w-64' : 'w-48'
                                                }`}>
                                                    <div className="p-2">
                                                        <h3 className="font-semibold text-gray-800 mb-2 text-sm">
                                                            {editor.username === username ? 'Your Results' : `${editor.username}'s Results`}
                                                        </h3>
                                                        <TestResultsDisplay result={submissionResults.get(editor.username)!} />
                                                    </div>
                                                </div>
                                            )}
                                        </div>
                                    );
                                })}
                            </div>
                        </>
                    )}
                </div>

                {/* Footer with current user info */}
                <div className="p-2 bg-gray-100 text-xs sm:text-sm text-gray-600 flex flex-col sm:flex-row justify-between gap-1">
                    <span>You are: <strong className="text-green-600">{username}</strong></span>
                    <span className="text-right">
                        {selectedLanguage.charAt(0).toUpperCase() + selectedLanguage.slice(1)} |
                        {currentProblem ? ` ${currentProblem.title}` : ' Loading...'} |
                        Users: {userEditorsArray.length}/{requiredPlayers}
                    </span>
                </div>
            </div>

            {/* Results Panel - Mobile Tab Content */}
            <div className={`${mobileTab === 'results' ? 'block' : 'hidden'} lg:hidden flex-1 overflow-auto bg-gray-50 p-4`}>
                <h2 className="text-lg font-bold text-gray-800 mb-4">Your Results</h2>
                {currentUserResult ? (
                    <TestResultsDisplay result={currentUserResult} />
                ) : (
                    <div className="text-center text-gray-500 py-8">
                        <p className="text-lg mb-2">No results yet</p>
                        <p className="text-sm">Submit your solution to see results here</p>
                    </div>
                )}

                {/* Other users' results on mobile */}
                {Array.from(submissionResults.entries())
                    .filter(([user]) => user !== username)
                    .map(([user, result]) => (
                        <div key={user} className="mt-6 pt-4 border-t border-gray-200">
                            <h3 className="text-md font-semibold text-gray-700 mb-2">{user}'s Results</h3>
                            <TestResultsDisplay result={result} />
                        </div>
                    ))
                }
            </div>
        </div>
    );
}
