// Room utilities
export function generateRoomCode(): string {
    const adjectives = ['SWIFT', 'SHARP', 'QUICK', 'SMART', 'BRAVE', 'FAST', 'COOL', 'EPIC'];
    const nouns = ['CODER', 'HACKER', 'NINJA', 'MASTER', 'WIZARD', 'GENIUS', 'HERO', 'CHAMP'];

    const adjective = adjectives[Math.floor(Math.random() * adjectives.length)];
    const noun = nouns[Math.floor(Math.random() * nouns.length)];
    const numbers = Math.floor(1000 + Math.random() * 9000); // 4-digit number

    return `${adjective}-${noun}-${numbers}`;
}

export function isValidRoomCode(code: string): boolean {
    // Format: ADJECTIVE-NOUN-NUMBERS (e.g., SWIFT-CODER-1234)
    const regex = /^[A-Z]+-[A-Z]+-\d{4}$/;
    return regex.test(code);
}

export function formatRoomCode(code: string): string {
    return code.toUpperCase().replace(/[^A-Z0-9]/g, '-');
}

// Room types
export interface RoomInfo {
    code: string;
    name: string;
    createdAt: number;
    createdBy: string;
    userCount: number;
    maxUsers: number;
    currentProblem?: string;
    isActive: boolean;
}

export interface RoomState {
    info: RoomInfo;
    users: string[];
    problemId?: string;
    submissions: Map<string, any>;
}