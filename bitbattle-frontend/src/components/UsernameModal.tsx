import { useState } from 'react';
import { useAuth } from '../contexts/AuthContext';

export default function UsernameModal() {
    const { setUsername } = useAuth();
    const [inputValue, setInputValue] = useState('');
    const [error, setError] = useState('');
    const [isSubmitting, setIsSubmitting] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');

        const username = inputValue.trim();

        if (!username) {
            setError('Username is required');
            return;
        }

        if (username.length > 20) {
            setError('Username must be 20 characters or less');
            return;
        }

        if (!/^[a-zA-Z0-9_-]+$/.test(username)) {
            setError('Only letters, numbers, underscores, and hyphens allowed');
            return;
        }

        setIsSubmitting(true);
        const success = await setUsername(username);
        setIsSubmitting(false);

        if (!success) {
            setError('Failed to set username. Try again.');
        }
    };

    return (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
            <div className="bg-gradient-to-br from-purple-900 to-indigo-900 rounded-2xl p-6 max-w-md w-full shadow-2xl">
                <h2 className="text-2xl font-bold text-white mb-2">Choose Your Username</h2>
                <p className="text-white/70 mb-6">This is how other players will see you in battles.</p>

                <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                        <input
                            type="text"
                            value={inputValue}
                            onChange={(e) => setInputValue(e.target.value)}
                            placeholder="Enter username..."
                            maxLength={20}
                            className="w-full px-4 py-3 bg-white/20 border border-white/30 rounded-lg text-white placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-purple-400 focus:border-transparent"
                            autoFocus
                        />
                        <p className="text-white/50 text-xs mt-1">
                            Letters, numbers, underscores, and hyphens only. Max 20 characters.
                        </p>
                        {error && (
                            <p className="text-red-400 text-sm mt-2">{error}</p>
                        )}
                    </div>

                    <button
                        type="submit"
                        disabled={isSubmitting || !inputValue.trim()}
                        className={`w-full py-3 px-4 rounded-lg font-medium transition-all ${
                            isSubmitting || !inputValue.trim()
                                ? 'bg-gray-600 text-gray-300 cursor-not-allowed'
                                : 'bg-purple-600 hover:bg-purple-700 text-white'
                        }`}
                    >
                        {isSubmitting ? 'Saving...' : 'Confirm Username'}
                    </button>
                </form>
            </div>
        </div>
    );
}
