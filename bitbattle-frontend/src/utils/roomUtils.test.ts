import { describe, it, expect } from 'vitest';
import { generateRoomCode, isValidRoomCode, formatRoomCode } from './roomUtils';

describe('roomUtils', () => {
    describe('generateRoomCode', () => {
        it('should generate a code in the correct format', () => {
            const code = generateRoomCode();
            expect(code).toMatch(/^[A-Z]+-[A-Z]+-\d{4}$/);
        });

        it('should generate codes with 4-digit numbers', () => {
            const code = generateRoomCode();
            const parts = code.split('-');
            expect(parts[2]).toMatch(/^\d{4}$/);
            const num = parseInt(parts[2], 10);
            expect(num).toBeGreaterThanOrEqual(1000);
            expect(num).toBeLessThan(10000);
        });

        it('should generate unique codes', () => {
            const codes = new Set<string>();
            // Generate 100 codes and check for uniqueness
            for (let i = 0; i < 100; i++) {
                codes.add(generateRoomCode());
            }
            // With random numbers from 1000-9999, we should have mostly unique codes
            expect(codes.size).toBeGreaterThan(90);
        });

        it('should use predefined adjectives', () => {
            const validAdjectives = ['SWIFT', 'SHARP', 'QUICK', 'SMART', 'BRAVE', 'FAST', 'COOL', 'EPIC'];
            const code = generateRoomCode();
            const adjective = code.split('-')[0];
            expect(validAdjectives).toContain(adjective);
        });

        it('should use predefined nouns', () => {
            const validNouns = ['CODER', 'HACKER', 'NINJA', 'MASTER', 'WIZARD', 'GENIUS', 'HERO', 'CHAMP'];
            const code = generateRoomCode();
            const noun = code.split('-')[1];
            expect(validNouns).toContain(noun);
        });
    });

    describe('isValidRoomCode', () => {
        it('should return true for valid codes', () => {
            expect(isValidRoomCode('SWIFT-CODER-1234')).toBe(true);
            expect(isValidRoomCode('QUICK-NINJA-0001')).toBe(true);
            expect(isValidRoomCode('EPIC-HERO-9999')).toBe(true);
        });

        it('should return false for lowercase codes', () => {
            expect(isValidRoomCode('swift-coder-1234')).toBe(false);
            expect(isValidRoomCode('Swift-Coder-1234')).toBe(false);
        });

        it('should return false for codes with wrong number of digits', () => {
            expect(isValidRoomCode('SWIFT-CODER-123')).toBe(false);
            expect(isValidRoomCode('SWIFT-CODER-12345')).toBe(false);
        });

        it('should return false for codes without dashes', () => {
            expect(isValidRoomCode('SWIFTCODER1234')).toBe(false);
            expect(isValidRoomCode('SWIFT_CODER_1234')).toBe(false);
        });

        it('should return false for codes with missing parts', () => {
            expect(isValidRoomCode('SWIFT-1234')).toBe(false);
            expect(isValidRoomCode('-CODER-1234')).toBe(false);
            expect(isValidRoomCode('SWIFT-CODER-')).toBe(false);
        });

        it('should return false for empty string', () => {
            expect(isValidRoomCode('')).toBe(false);
        });
    });

    describe('formatRoomCode', () => {
        it('should convert to uppercase', () => {
            expect(formatRoomCode('swift-coder-1234')).toBe('SWIFT-CODER-1234');
            expect(formatRoomCode('Swift-Coder-1234')).toBe('SWIFT-CODER-1234');
        });

        it('should replace non-alphanumeric characters with dashes', () => {
            expect(formatRoomCode('swift_coder_1234')).toBe('SWIFT-CODER-1234');
            expect(formatRoomCode('swift.coder.1234')).toBe('SWIFT-CODER-1234');
            expect(formatRoomCode('swift coder 1234')).toBe('SWIFT-CODER-1234');
        });

        it('should handle already valid codes', () => {
            expect(formatRoomCode('SWIFT-CODER-1234')).toBe('SWIFT-CODER-1234');
        });
    });
});
