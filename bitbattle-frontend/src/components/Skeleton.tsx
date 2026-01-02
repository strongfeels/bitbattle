interface SkeletonProps {
    className?: string;
    variant?: 'text' | 'circular' | 'rectangular';
    width?: string | number;
    height?: string | number;
}

export function Skeleton({
    className = '',
    variant = 'text',
    width,
    height
}: SkeletonProps) {
    const baseClasses = 'skeleton animate-pulse';

    const variantClasses = {
        text: 'rounded',
        circular: 'rounded-full',
        rectangular: 'rounded-lg',
    };

    const style: React.CSSProperties = {};
    if (width) style.width = typeof width === 'number' ? `${width}px` : width;
    if (height) style.height = typeof height === 'number' ? `${height}px` : height;

    return (
        <div
            className={`${baseClasses} ${variantClasses[variant]} ${className}`}
            style={style}
            aria-hidden="true"
        />
    );
}

// Pre-built skeleton patterns for common use cases
export function LeaderboardSkeleton() {
    return (
        <div className="space-y-2" role="status" aria-label="Loading leaderboard">
            {Array.from({ length: 10 }).map((_, i) => (
                <div key={i} className="flex items-center gap-4 px-4 py-3">
                    <Skeleton width={32} height={20} variant="rectangular" />
                    <Skeleton width={32} height={32} variant="circular" />
                    <Skeleton className="flex-1 h-5" />
                    <Skeleton width={48} height={20} variant="rectangular" />
                    <Skeleton width={48} height={20} variant="rectangular" />
                    <Skeleton width={48} height={20} variant="rectangular" />
                </div>
            ))}
            <span className="sr-only">Loading leaderboard data...</span>
        </div>
    );
}

export function ProfileSkeleton() {
    return (
        <div className="space-y-4" role="status" aria-label="Loading profile">
            {/* Header skeleton */}
            <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-4">
                <div className="flex items-center gap-4">
                    <Skeleton width={64} height={64} variant="circular" />
                    <div className="space-y-2">
                        <Skeleton width={160} height={24} variant="rectangular" />
                        <Skeleton width={120} height={16} variant="rectangular" />
                    </div>
                </div>
            </div>

            {/* Stats skeleton */}
            <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-4">
                <Skeleton width={80} height={16} className="mb-3" />
                <div className="grid grid-cols-4 gap-4">
                    {Array.from({ length: 4 }).map((_, i) => (
                        <div key={i} className="text-center space-y-1">
                            <Skeleton width={48} height={32} className="mx-auto" variant="rectangular" />
                            <Skeleton width={40} height={12} className="mx-auto" />
                        </div>
                    ))}
                </div>
            </div>

            {/* Ranked ratings skeleton */}
            <div className="bg-zinc-800 border border-zinc-700 rounded-lg p-4">
                <Skeleton width={120} height={16} className="mb-3" />
                <div className="space-y-3">
                    {Array.from({ length: 3 }).map((_, i) => (
                        <div key={i} className="flex items-center justify-between py-2 border-b border-zinc-700 last:border-0">
                            <div className="flex items-center gap-3">
                                <Skeleton width={48} height={20} variant="rectangular" />
                                <Skeleton width={56} height={28} variant="rectangular" />
                            </div>
                            <Skeleton width={80} height={20} variant="rectangular" />
                        </div>
                    ))}
                </div>
            </div>

            {/* Recent games skeleton */}
            <div className="bg-zinc-800 border border-zinc-700 rounded-lg overflow-hidden">
                <div className="px-4 py-3 border-b border-zinc-700">
                    <Skeleton width={100} height={16} />
                </div>
                <div className="divide-y divide-zinc-700">
                    {Array.from({ length: 5 }).map((_, i) => (
                        <div key={i} className="flex items-center gap-4 px-4 py-3">
                            <Skeleton className="flex-1 h-4" />
                            <Skeleton width={48} height={16} variant="rectangular" />
                            <Skeleton width={56} height={16} variant="rectangular" />
                            <Skeleton width={48} height={16} variant="rectangular" />
                        </div>
                    ))}
                </div>
            </div>
            <span className="sr-only">Loading profile data...</span>
        </div>
    );
}

export function GameHistorySkeleton() {
    return (
        <div className="divide-y divide-zinc-700" role="status" aria-label="Loading game history">
            {Array.from({ length: 5 }).map((_, i) => (
                <div key={i} className="flex items-center gap-4 px-4 py-3">
                    <Skeleton className="flex-1 h-4" />
                    <Skeleton width={48} height={16} variant="rectangular" />
                    <Skeleton width={56} height={16} variant="rectangular" />
                    <Skeleton width={48} height={16} variant="rectangular" />
                </div>
            ))}
            <span className="sr-only">Loading game history...</span>
        </div>
    );
}
