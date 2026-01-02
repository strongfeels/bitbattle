/**
 * Standard API error response format from the backend
 */
export interface ApiError {
    /** Error code for programmatic handling (e.g., "VALIDATION_ERROR", "NOT_FOUND") */
    code: string;
    /** Human-readable error message */
    message: string;
    /** Optional field that caused the error (for validation errors) */
    field?: string;
    /** Optional additional details */
    details?: Record<string, unknown>;
}

/**
 * Error codes returned by the backend
 */
export const ErrorCodes = {
    // Authentication errors
    UNAUTHORIZED: 'UNAUTHORIZED',
    INVALID_TOKEN: 'INVALID_TOKEN',
    TOKEN_EXPIRED: 'TOKEN_EXPIRED',
    SESSION_REVOKED: 'SESSION_REVOKED',

    // Authorization errors
    FORBIDDEN: 'FORBIDDEN',

    // Validation errors
    VALIDATION_ERROR: 'VALIDATION_ERROR',
    INVALID_INPUT: 'INVALID_INPUT',

    // Resource errors
    NOT_FOUND: 'NOT_FOUND',
    ALREADY_EXISTS: 'ALREADY_EXISTS',

    // Server errors
    DATABASE_ERROR: 'DATABASE_ERROR',
    EXTERNAL_SERVICE_ERROR: 'EXTERNAL_SERVICE_ERROR',
    RATE_LIMIT_EXCEEDED: 'RATE_LIMIT_EXCEEDED',
    INTERNAL_ERROR: 'INTERNAL_ERROR',
    BAD_REQUEST: 'BAD_REQUEST',
} as const;

export type ErrorCode = typeof ErrorCodes[keyof typeof ErrorCodes];

/**
 * Custom error class for API errors with typed error codes
 */
export class ApiErrorResponse extends Error {
    public readonly code: string;
    public readonly field?: string;
    public readonly details?: Record<string, unknown>;
    public readonly statusCode: number;

    constructor(error: ApiError, statusCode: number) {
        super(error.message);
        this.name = 'ApiErrorResponse';
        this.code = error.code;
        this.field = error.field;
        this.details = error.details;
        this.statusCode = statusCode;
    }

    /**
     * Check if this is an authentication error
     */
    isAuthError(): boolean {
        const authErrors: string[] = [
            ErrorCodes.UNAUTHORIZED,
            ErrorCodes.INVALID_TOKEN,
            ErrorCodes.TOKEN_EXPIRED,
            ErrorCodes.SESSION_REVOKED,
        ];
        return authErrors.includes(this.code);
    }

    /**
     * Check if this is a validation error
     */
    isValidationError(): boolean {
        const validationErrors: string[] = [
            ErrorCodes.VALIDATION_ERROR,
            ErrorCodes.INVALID_INPUT,
        ];
        return validationErrors.includes(this.code);
    }

    /**
     * Check if this is a not found error
     */
    isNotFound(): boolean {
        return this.code === ErrorCodes.NOT_FOUND;
    }

    /**
     * Check if this is a rate limit error
     */
    isRateLimited(): boolean {
        return this.code === ErrorCodes.RATE_LIMIT_EXCEEDED;
    }

    /**
     * Get a user-friendly error message
     */
    getUserMessage(): string {
        switch (this.code) {
            case ErrorCodes.TOKEN_EXPIRED:
            case ErrorCodes.SESSION_REVOKED:
                return 'Your session has expired. Please log in again.';
            case ErrorCodes.RATE_LIMIT_EXCEEDED:
                return 'Too many requests. Please wait a moment and try again.';
            case ErrorCodes.FORBIDDEN:
                return 'You do not have permission to perform this action.';
            case ErrorCodes.NOT_FOUND:
                return 'The requested resource was not found.';
            case ErrorCodes.DATABASE_ERROR:
            case ErrorCodes.INTERNAL_ERROR:
                return 'An unexpected error occurred. Please try again later.';
            default:
                return this.message;
        }
    }
}

/**
 * Parse an API error response from a fetch Response
 */
export async function parseApiError(response: Response): Promise<ApiErrorResponse> {
    try {
        const data = await response.json();
        // Check if it's our standard error format
        if (data.code && data.message) {
            return new ApiErrorResponse(data as ApiError, response.status);
        }
        // Legacy error format
        if (data.error) {
            return new ApiErrorResponse(
                { code: 'UNKNOWN_ERROR', message: data.error },
                response.status
            );
        }
        // Unknown format
        return new ApiErrorResponse(
            { code: 'UNKNOWN_ERROR', message: 'An unexpected error occurred' },
            response.status
        );
    } catch {
        // Could not parse JSON
        return new ApiErrorResponse(
            { code: 'NETWORK_ERROR', message: `Request failed with status ${response.status}` },
            response.status
        );
    }
}

/**
 * Type guard to check if an error is an ApiErrorResponse
 */
export function isApiError(error: unknown): error is ApiErrorResponse {
    return error instanceof ApiErrorResponse;
}

/**
 * Get a user-friendly message from any error
 */
export function getErrorMessage(error: unknown): string {
    if (isApiError(error)) {
        return error.getUserMessage();
    }
    if (error instanceof Error) {
        return error.message;
    }
    return 'An unexpected error occurred';
}
