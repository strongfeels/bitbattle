import { useNavigate } from 'react-router-dom';

interface RatingChange {
    old_rating: number;
    new_rating: number;
    change: number;
}

interface GameOverData {
    winner: string;
    solve_time_ms: number;
    problem_id: string;
    difficulty: string;
    game_mode: string;
    rating_changes: Record<string, RatingChange>;
    players: string[];
}

interface Props {
    data: GameOverData;
    currentUser: string;
    onPlayAgain: () => void;
}

export default function GameOverScreen({ data, currentUser, onPlayAgain }: Props) {
    const navigate = useNavigate();
    const isWinner = data.winner === currentUser;
    const myRatingChange = data.rating_changes[currentUser];

    const formatTime = (ms: number): string => {
        const seconds = Math.floor(ms / 1000);
        const minutes = Math.floor(seconds / 60);
        const secs = seconds % 60;
        const millis = ms % 1000;
        if (minutes > 0) {
            return `${minutes}:${secs.toString().padStart(2, '0')}.${millis.toString().padStart(3, '0')}`;
        }
        return `${secs}.${millis.toString().padStart(3, '0')}s`;
    };

    const getDifficultyColor = (diff: string): string => {
        switch (diff.toLowerCase()) {
            case 'easy':
                return 'text-green-400';
            case 'medium':
                return 'text-yellow-400';
            case 'hard':
                return 'text-red-400';
            default:
                return 'text-purple-400';
        }
    };

    const getRatingChangeColor = (change: number): string => {
        if (change > 0) return 'text-green-400';
        if (change < 0) return 'text-red-400';
        return 'text-zinc-400';
    };

    const getRatingChangeSymbol = (change: number): string => {
        if (change > 0) return '+';
        return '';
    };

    return (
        <div className="fixed inset-0 bg-black/90 flex items-center justify-center z-50 animate-fadeIn">
            <div className="bg-zinc-900 border border-zinc-700 rounded-xl p-8 max-w-md w-full mx-4 shadow-2xl">
                {/* Victory/Defeat Header */}
                <div className="text-center mb-6">
                    {isWinner ? (
                        <>
                            <div className="w-20 h-20 rounded-full bg-gradient-to-br from-yellow-400 to-amber-600 flex items-center justify-center mx-auto mb-4 animate-bounce">
                                <svg className="w-10 h-10 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
                                </svg>
                            </div>
                            <h2 className="text-4xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-yellow-400 to-amber-500 mb-2">
                                Victory!
                            </h2>
                            <p className="text-zinc-400">You solved the problem first!</p>
                        </>
                    ) : (
                        <>
                            <div className="w-20 h-20 rounded-full bg-zinc-800 flex items-center justify-center mx-auto mb-4">
                                <svg className="w-10 h-10 text-zinc-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                            </div>
                            <h2 className="text-4xl font-bold text-zinc-400 mb-2">
                                Defeat
                            </h2>
                            <p className="text-zinc-500">
                                <span className="text-white font-semibold">{data.winner}</span> solved it first
                            </p>
                        </>
                    )}
                </div>

                {/* Game Stats */}
                <div className="bg-zinc-800/50 rounded-lg p-4 mb-6 space-y-3">
                    <div className="flex justify-between items-center">
                        <span className="text-zinc-500">Problem</span>
                        <span className="text-white font-medium">{data.problem_id}</span>
                    </div>
                    <div className="flex justify-between items-center">
                        <span className="text-zinc-500">Difficulty</span>
                        <span className={`font-medium capitalize ${getDifficultyColor(data.difficulty)}`}>
                            {data.difficulty}
                        </span>
                    </div>
                    <div className="flex justify-between items-center">
                        <span className="text-zinc-500">Winning Time</span>
                        <span className="text-white font-medium">{formatTime(data.solve_time_ms)}</span>
                    </div>
                    <div className="flex justify-between items-center">
                        <span className="text-zinc-500">Mode</span>
                        <span className={`font-medium capitalize ${data.game_mode === 'ranked' ? 'text-amber-400' : 'text-blue-400'}`}>
                            {data.game_mode}
                        </span>
                    </div>
                </div>

                {/* Rating Change (only for ranked) */}
                {data.game_mode === 'ranked' && myRatingChange && (
                    <div className="bg-zinc-800/50 rounded-lg p-4 mb-6">
                        <h3 className="text-sm font-medium text-zinc-500 mb-3 uppercase tracking-wide">Rating Change</h3>
                        <div className="flex items-center justify-center gap-4">
                            <div className="text-center">
                                <p className="text-2xl font-bold text-zinc-400">{myRatingChange.old_rating}</p>
                                <p className="text-xs text-zinc-600">Previous</p>
                            </div>
                            <div className="flex items-center">
                                <svg className="w-6 h-6 text-zinc-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
                                </svg>
                            </div>
                            <div className="text-center">
                                <p className={`text-2xl font-bold ${isWinner ? 'text-green-400' : 'text-red-400'}`}>
                                    {myRatingChange.new_rating}
                                </p>
                                <p className="text-xs text-zinc-600">New</p>
                            </div>
                            <div className={`text-lg font-bold ${getRatingChangeColor(myRatingChange.change)}`}>
                                ({getRatingChangeSymbol(myRatingChange.change)}{myRatingChange.change})
                            </div>
                        </div>
                    </div>
                )}

                {/* Players Summary */}
                <div className="bg-zinc-800/50 rounded-lg p-4 mb-6">
                    <h3 className="text-sm font-medium text-zinc-500 mb-3 uppercase tracking-wide">Players</h3>
                    <div className="space-y-2">
                        {data.players.map((player, index) => {
                            const playerRating = data.rating_changes[player];
                            const isPlayerWinner = player === data.winner;
                            return (
                                <div
                                    key={player}
                                    className={`flex items-center justify-between p-2 rounded ${
                                        isPlayerWinner ? 'bg-amber-500/10 border border-amber-500/30' : 'bg-zinc-700/30'
                                    }`}
                                >
                                    <div className="flex items-center gap-2">
                                        <span className="text-zinc-500 text-sm w-4">{index + 1}.</span>
                                        <span className={`font-medium ${isPlayerWinner ? 'text-amber-400' : 'text-zinc-300'}`}>
                                            {player}
                                            {player === currentUser && <span className="text-zinc-500 text-sm ml-1">(you)</span>}
                                        </span>
                                        {isPlayerWinner && (
                                            <svg className="w-4 h-4 text-amber-400" fill="currentColor" viewBox="0 0 20 20">
                                                <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                                            </svg>
                                        )}
                                    </div>
                                    {data.game_mode === 'ranked' && playerRating && (
                                        <span className={`text-sm font-medium ${getRatingChangeColor(playerRating.change)}`}>
                                            {getRatingChangeSymbol(playerRating.change)}{playerRating.change}
                                        </span>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                </div>

                {/* Action Buttons */}
                <div className="flex gap-3">
                    <button
                        onClick={() => navigate('/')}
                        className="flex-1 py-3 rounded-lg text-sm font-medium border border-zinc-700 text-zinc-300 hover:bg-zinc-800 transition-colors"
                    >
                        Back to Lobby
                    </button>
                    <button
                        onClick={onPlayAgain}
                        className="flex-1 py-3 rounded-lg text-sm font-medium bg-blue-600 text-white hover:bg-blue-700 transition-colors"
                    >
                        Play Again
                    </button>
                </div>
            </div>
        </div>
    );
}
