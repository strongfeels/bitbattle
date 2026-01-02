/**
 * Design tokens for BitBattle
 * These values define the visual language of the application
 */

// Color palette
export const colors = {
    // Brand colors
    brand: {
        primary: '#3b82f6',    // Blue-500
        secondary: '#8b5cf6',  // Purple-500
        accent: '#f59e0b',     // Amber-500
    },

    // Game mode colors
    mode: {
        casual: '#3b82f6',     // Blue for casual
        ranked: '#f59e0b',     // Amber for ranked
    },

    // Difficulty colors
    difficulty: {
        easy: '#22c55e',       // Green-500
        medium: '#eab308',     // Yellow-500
        hard: '#ef4444',       // Red-500
        random: '#a855f7',     // Purple-500
    },

    // Status colors
    status: {
        success: '#22c55e',    // Green-500
        error: '#ef4444',      // Red-500
        warning: '#f59e0b',    // Amber-500
        info: '#3b82f6',       // Blue-500
    },

    // Placement colors
    placement: {
        first: '#facc15',      // Yellow-400 (gold)
        second: '#d4d4d8',     // Zinc-300 (silver)
        third: '#d97706',      // Amber-600 (bronze)
    },

    // Background colors (zinc scale)
    background: {
        primary: '#18181b',    // Zinc-900
        secondary: '#27272a',  // Zinc-800
        tertiary: '#3f3f46',   // Zinc-700
        elevated: '#52525b',   // Zinc-600
    },

    // Text colors
    text: {
        primary: '#ffffff',
        secondary: '#a1a1aa',  // Zinc-400
        tertiary: '#71717a',   // Zinc-500
        muted: '#52525b',      // Zinc-600
    },

    // Border colors
    border: {
        default: '#3f3f46',    // Zinc-700
        light: '#52525b',      // Zinc-600
        focus: '#3b82f6',      // Blue-500
    },
} as const;

// Spacing scale (in Tailwind units)
export const spacing = {
    xs: '0.25rem',   // 1
    sm: '0.5rem',    // 2
    md: '1rem',      // 4
    lg: '1.5rem',    // 6
    xl: '2rem',      // 8
    '2xl': '3rem',   // 12
    '3xl': '4rem',   // 16
} as const;

// Border radius
export const borderRadius = {
    none: '0',
    sm: '0.25rem',   // rounded-sm
    md: '0.375rem',  // rounded
    lg: '0.5rem',    // rounded-lg
    xl: '0.75rem',   // rounded-xl
    full: '9999px',  // rounded-full
} as const;

// Font sizes
export const fontSize = {
    xs: '0.75rem',   // 12px
    sm: '0.875rem',  // 14px
    base: '1rem',    // 16px
    lg: '1.125rem',  // 18px
    xl: '1.25rem',   // 20px
    '2xl': '1.5rem', // 24px
    '3xl': '1.875rem', // 30px
} as const;

// Font weights
export const fontWeight = {
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
} as const;

// Animation durations
export const duration = {
    fast: '150ms',
    normal: '200ms',
    slow: '300ms',
    slower: '500ms',
} as const;

// Z-index scale
export const zIndex = {
    dropdown: 10,
    sticky: 20,
    fixed: 30,
    modalBackdrop: 40,
    modal: 50,
    popover: 60,
    tooltip: 70,
    toast: 80,
} as const;

// Breakpoints (matching Tailwind defaults)
export const breakpoints = {
    sm: '640px',
    md: '768px',
    lg: '1024px',
    xl: '1280px',
    '2xl': '1536px',
} as const;

// Helper function to get difficulty color
export function getDifficultyColor(difficulty: 'easy' | 'medium' | 'hard' | 'random'): string {
    return colors.difficulty[difficulty];
}

// Helper function to get mode color
export function getModeColor(mode: 'casual' | 'ranked'): string {
    return colors.mode[mode];
}

// Helper function to get placement color
export function getPlacementColor(placement: number): string {
    if (placement === 1) return colors.placement.first;
    if (placement === 2) return colors.placement.second;
    if (placement === 3) return colors.placement.third;
    return colors.text.secondary;
}

// Tailwind class helpers
export const tw = {
    // Difficulty badge classes
    difficultyBadge: {
        easy: 'text-green-600 bg-green-100',
        medium: 'text-yellow-600 bg-yellow-100',
        hard: 'text-red-600 bg-red-100',
        random: 'text-purple-600 bg-purple-100',
    },

    // Mode badge classes
    modeBadge: {
        casual: 'text-blue-400',
        ranked: 'text-amber-400',
    },

    // Placement text classes
    placementText: {
        1: 'text-yellow-400',
        2: 'text-zinc-300',
        3: 'text-amber-600',
        default: 'text-zinc-400',
    },

    // Button variants
    button: {
        primary: 'bg-blue-600 hover:bg-blue-500 text-white',
        secondary: 'bg-zinc-700 hover:bg-zinc-600 text-white',
        success: 'bg-green-600 hover:bg-green-500 text-white',
        danger: 'bg-red-600 hover:bg-red-500 text-white',
        ghost: 'bg-transparent hover:bg-zinc-700 text-zinc-300',
    },
} as const;
