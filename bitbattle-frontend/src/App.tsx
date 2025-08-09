import { useState } from 'react';
import CollaborativeEditor from './components/CollaborativeEditor.tsx';

function App() {
    const [username, setUsername] = useState<string>('');
    const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);

    const handleLogin = (e: React.FormEvent) => {
        e.preventDefault();
        if (username.trim()) {
            setIsLoggedIn(true);
        }
    };

    if (!isLoggedIn) {
        return (
            <div className="min-h-screen bg-gray-900 flex items-center justify-center">
                <div className="bg-white p-8 rounded-lg shadow-lg w-96">
                    <h1 className="text-2xl font-bold text-center mb-6">BitBattle</h1>
                    <form onSubmit={handleLogin}>
                        <div className="mb-4">
                            <label htmlFor="username" className="block text-sm font-medium text-gray-700 mb-2">
                                Enter your username
                            </label>
                            <input
                                type="text"
                                id="username"
                                value={username}
                                onChange={(e) => setUsername(e.target.value)}
                                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                placeholder="Your username..."
                                required
                            />
                        </div>
                        <button
                            type="submit"
                            className="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        >
                            Join Coding Session
                        </button>
                    </form>
                </div>
            </div>
        );
    }

    return (
        <div className="h-screen">
            <CollaborativeEditor
                username={username}
                roomId="default"
            />
        </div>
    );
}

export default App;