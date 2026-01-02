import { parseApiError, ApiErrorResponse, ErrorCodes } from '../types/error';

// Use environment variable or fall back to localhost for development
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:4000';

// Export for use in WebSocket connections
export const getApiBaseUrl = () => API_BASE_URL;
export const getWsBaseUrl = () => API_BASE_URL.replace(/^http/, 'ws');

// Token storage keys
const ACCESS_TOKEN_KEY = 'bitbattle_access_token';
const REFRESH_TOKEN_KEY = 'bitbattle_refresh_token';
const LEGACY_TOKEN_KEY = 'bitbattle_token'; // For backwards compatibility

// Track if we're currently refreshing to prevent multiple refresh calls
let isRefreshing = false;
let refreshPromise: Promise<boolean> | null = null;

export async function apiFetch<T>(
    endpoint: string,
    options: RequestInit = {},
    skipRefresh: boolean = false
): Promise<T> {
    const token = getAccessToken();

    const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        ...options.headers as Record<string, string>,
    };

    if (token) {
        headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
        ...options,
        headers,
    });

    // If unauthorized and we have a refresh token, try to refresh
    if (response.status === 401 && !skipRefresh) {
        const refreshToken = getRefreshToken();
        if (refreshToken) {
            const refreshed = await refreshAccessToken();
            if (refreshed) {
                // Retry the original request with new token
                return apiFetch<T>(endpoint, options, true);
            }
        }
        // Clear tokens if refresh failed
        clearAuthTokens();
        throw new ApiErrorResponse(
            { code: ErrorCodes.SESSION_REVOKED, message: 'Session expired. Please login again.' },
            401
        );
    }

    if (!response.ok) {
        const error = await parseApiError(response);
        throw error;
    }

    return response.json();
}

// Refresh the access token using the refresh token
async function refreshAccessToken(): Promise<boolean> {
    // If already refreshing, wait for that to complete
    if (isRefreshing && refreshPromise) {
        return refreshPromise;
    }

    isRefreshing = true;
    refreshPromise = (async () => {
        try {
            const refreshToken = getRefreshToken();
            if (!refreshToken) {
                return false;
            }

            const response = await fetch(`${API_BASE_URL}/auth/refresh`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ refresh_token: refreshToken }),
            });

            if (!response.ok) {
                console.error('Token refresh failed:', response.status);
                return false;
            }

            const data = await response.json();
            if (data.access_token) {
                setAccessToken(data.access_token);
                console.log('Access token refreshed successfully');
                return true;
            }

            return false;
        } catch (error) {
            console.error('Token refresh error:', error);
            return false;
        } finally {
            isRefreshing = false;
            refreshPromise = null;
        }
    })();

    return refreshPromise;
}

// Get access token
export function getAccessToken(): string | null {
    return localStorage.getItem(ACCESS_TOKEN_KEY) || localStorage.getItem(LEGACY_TOKEN_KEY);
}

// Get refresh token
export function getRefreshToken(): string | null {
    return localStorage.getItem(REFRESH_TOKEN_KEY);
}

// Legacy function - get any auth token
export function getAuthToken(): string | null {
    return getAccessToken();
}

// Set access token
export function setAccessToken(token: string): void {
    localStorage.setItem(ACCESS_TOKEN_KEY, token);
}

// Set refresh token
export function setRefreshToken(token: string): void {
    localStorage.setItem(REFRESH_TOKEN_KEY, token);
}

// Set both tokens
export function setAuthTokens(accessToken: string, refreshToken: string): void {
    setAccessToken(accessToken);
    setRefreshToken(refreshToken);
    // Remove legacy token if exists
    localStorage.removeItem(LEGACY_TOKEN_KEY);
}

// Legacy function - set single token
export function setAuthToken(token: string): void {
    setAccessToken(token);
}

// Clear all auth tokens
export function clearAuthTokens(): void {
    localStorage.removeItem(ACCESS_TOKEN_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    localStorage.removeItem(LEGACY_TOKEN_KEY);
}

// Legacy function
export function clearAuthToken(): void {
    clearAuthTokens();
}

// Logout - revoke refresh token on server
export async function logoutApi(): Promise<void> {
    const refreshToken = getRefreshToken();
    if (refreshToken) {
        try {
            await fetch(`${API_BASE_URL}/auth/logout`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ refresh_token: refreshToken }),
            });
        } catch (error) {
            console.error('Logout API error:', error);
        }
    }
    clearAuthTokens();
}

export function getGoogleAuthUrl(): string {
    return `${API_BASE_URL}/auth/google`;
}
