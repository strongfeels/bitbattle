
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

interface Props {
    problem: Problem | null;
    timeRemaining?: number;
}

export default function ProblemPanel({ problem, timeRemaining }: Props) {
    if (!problem) {
        return (
            <div className="h-full bg-gray-50 p-4 md:p-6 flex items-center justify-center">
                <div className="text-center">
                    <div className="text-gray-400 text-lg mb-2">Loading...</div>
                    <p className="text-gray-600 text-sm md:text-base">Waiting for problem assignment...</p>
                </div>
            </div>
        );
    }

    const getDifficultyColor = () => {
        switch (problem.difficulty) {
            case 'Easy': return 'text-green-600 bg-green-100';
            case 'Medium': return 'text-yellow-600 bg-yellow-100';
            case 'Hard': return 'text-red-600 bg-red-100';
        }
    };

    const formatTimeRemaining = (seconds: number) => {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    };

    return (
        <div className="h-full bg-white flex flex-col">
            {/* Header */}
            <div className="border-b border-gray-200 p-3 md:p-4">
                <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 mb-2">
                    <h1 className="text-lg md:text-xl font-bold text-gray-900">{problem.title}</h1>
                    <div className="flex items-center space-x-2">
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${getDifficultyColor()}`}>
                            {problem.difficulty}
                        </span>
                        {timeRemaining !== undefined && (
                            <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                                timeRemaining < 300 ? 'text-red-600 bg-red-100' : 'text-blue-600 bg-blue-100'
                            }`}>
                                {formatTimeRemaining(timeRemaining)}
                            </span>
                        )}
                    </div>
                </div>

                {/* Tags */}
                <div className="flex flex-wrap gap-1">
                    {problem.tags.map(tag => (
                        <span
                            key={tag}
                            className="px-2 py-0.5 md:py-1 bg-gray-100 text-gray-600 text-xs rounded"
                        >
                            {tag}
                        </span>
                    ))}
                </div>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-y-auto p-3 md:p-4">
                {/* Description */}
                <div className="mb-4 md:mb-6">
                    <h2 className="text-base md:text-lg font-semibold text-gray-800 mb-2">Problem Description</h2>
                    <div className="prose prose-sm max-w-none">
                        {problem.description.split('\n').map((paragraph, index) => (
                            <p key={index} className="mb-2 text-sm md:text-base text-gray-700">
                                {paragraph}
                            </p>
                        ))}
                    </div>
                </div>

                {/* Examples */}
                {problem.examples.length > 0 && (
                    <div className="mb-4 md:mb-6">
                        <h2 className="text-base md:text-lg font-semibold text-gray-800 mb-2">Examples</h2>
                        {problem.examples.map((example, index) => (
                            <div key={index} className="mb-3 md:mb-4 p-2 md:p-3 bg-gray-50 rounded-lg">
                                <div className="font-medium text-gray-700 mb-1 text-sm md:text-base">
                                    Example {index + 1}:
                                </div>

                                <div className="mb-2">
                                    <span className="text-xs md:text-sm font-medium text-gray-600">Input: </span>
                                    <code className="text-xs md:text-sm bg-gray-200 px-1 py-0.5 rounded break-all">
                                        {example.input}
                                    </code>
                                </div>

                                <div className="mb-2">
                                    <span className="text-xs md:text-sm font-medium text-gray-600">Output: </span>
                                    <code className="text-xs md:text-sm bg-gray-200 px-1 py-0.5 rounded break-all">
                                        {example.expected_output}
                                    </code>
                                </div>

                                {example.explanation && (
                                    <div className="text-xs md:text-sm text-gray-600">
                                        <span className="font-medium">Explanation: </span>
                                        {example.explanation}
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                )}

                {/* Constraints/Notes */}
                <div className="bg-blue-50 p-2 md:p-3 rounded-lg">
                    <h3 className="font-medium text-blue-800 mb-1 text-sm md:text-base">Tips</h3>
                    <ul className="text-xs md:text-sm text-blue-700 space-y-1">
                        <li>Test your solution with the examples</li>
                        <li>Consider edge cases and error handling</li>
                        <li>Think about time and space complexity</li>
                        {problem.time_limit_minutes && (
                            <li>You have {problem.time_limit_minutes} minutes to complete this</li>
                        )}
                    </ul>
                </div>
            </div>
        </div>
    );
}
