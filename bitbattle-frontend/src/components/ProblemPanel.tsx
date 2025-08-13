import React from 'react';

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
    selectedLanguage: string;
    onLanguageChange: (language: string) => void;
}

export default function ProblemPanel({ problem, timeRemaining, selectedLanguage, onLanguageChange }: Props) {
    if (!problem) {
        return (
            <div className="h-full bg-gray-50 p-6 flex items-center justify-center">
                <div className="text-center">
                    <div className="text-gray-400 text-lg mb-2">⏳</div>
                    <p className="text-gray-600">Waiting for problem assignment...</p>
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
            <div className="border-b border-gray-200 p-4">
                <div className="flex items-center justify-between mb-2">
                    <h1 className="text-xl font-bold text-gray-900">{problem.title}</h1>
                    <div className="flex items-center space-x-2">
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${getDifficultyColor()}`}>
                            {problem.difficulty}
                        </span>
                        {timeRemaining !== undefined && (
                            <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                                timeRemaining < 300 ? 'text-red-600 bg-red-100' : 'text-blue-600 bg-blue-100'
                            }`}>
                                ⏱️ {formatTimeRemaining(timeRemaining)}
                            </span>
                        )}
                    </div>
                </div>

                {/* Language Selector */}
                <div className="mb-3">
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                        Programming Language:
                    </label>
                    <select
                        value={selectedLanguage}
                        onChange={(e) => onLanguageChange(e.target.value)}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                    >
                        <option value="javascript">JavaScript</option>
                        <option value="python">Python</option>
                        <option value="java">Java</option>
                    </select>
                </div>

                {/* Tags */}
                <div className="flex flex-wrap gap-1">
                    {problem.tags.map(tag => (
                        <span
                            key={tag}
                            className="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded"
                        >
                            {tag}
                        </span>
                    ))}
                </div>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-y-auto p-4">
                {/* Description */}
                <div className="mb-6">
                    <h2 className="text-lg font-semibold text-gray-800 mb-2">Problem Description</h2>
                    <div className="prose prose-sm max-w-none">
                        {problem.description.split('\n').map((paragraph, index) => (
                            <p key={index} className="mb-2 text-gray-700">
                                {paragraph}
                            </p>
                        ))}
                    </div>
                </div>

                {/* Examples */}
                {problem.examples.length > 0 && (
                    <div className="mb-6">
                        <h2 className="text-lg font-semibold text-gray-800 mb-2">Examples</h2>
                        {problem.examples.map((example, index) => (
                            <div key={index} className="mb-4 p-3 bg-gray-50 rounded-lg">
                                <div className="font-medium text-gray-700 mb-1">
                                    Example {index + 1}:
                                </div>

                                <div className="mb-2">
                                    <span className="text-sm font-medium text-gray-600">Input: </span>
                                    <code className="text-sm bg-gray-200 px-1 py-0.5 rounded">
                                        {example.input}
                                    </code>
                                </div>

                                <div className="mb-2">
                                    <span className="text-sm font-medium text-gray-600">Output: </span>
                                    <code className="text-sm bg-gray-200 px-1 py-0.5 rounded">
                                        {example.expected_output}
                                    </code>
                                </div>

                                {example.explanation && (
                                    <div className="text-sm text-gray-600">
                                        <span className="font-medium">Explanation: </span>
                                        {example.explanation}
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                )}

                {/* Constraints/Notes */}
                <div className="bg-blue-50 p-3 rounded-lg">
                    <h3 className="font-medium text-blue-800 mb-1">💡 Tips</h3>
                    <ul className="text-sm text-blue-700 space-y-1">
                        <li>• Test your solution with the provided examples</li>
                        <li>• Consider edge cases and error handling</li>
                        <li>• Think about time and space complexity</li>
                        {problem.time_limit_minutes && (
                            <li>• You have {problem.time_limit_minutes} minutes to complete this problem</li>
                        )}
                    </ul>
                </div>
            </div>
        </div>
    );
}