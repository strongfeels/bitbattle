import { createContext, useContext, useState, useEffect } from 'react';
import type { ReactNode } from 'react';
import type { User } from '../types/auth';
import { getErrorMessage } from '../types/error';
import { apiFetch, setAuthTokens, clearAuthTokens, getAccessToken, logoutApi, getGoogleAuthUrl } from '../utils/api';

interface AuthContextType {
    user: User | null;
    isLoading: boolean;
    isAuthenticated: boolean;
    isNewUser: boolean;
    error: string | null;
    login: () => void;
    logout: () => Promise<void>;
    setUsername: (username: string) => Promise<{ success: boolean; error?: string }>;
    clearNewUser: () => void;
    clearError: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
    const [user, setUser] = useState<User | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isNewUser, setIsNewUser] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        // Check for tokens in URL (OAuth callback)
        const urlParams = new URLSearchParams(window.location.search);
        const accessToken = urlParams.get('access_token');
        const refreshToken = urlParams.get('refresh_token');
        const legacyToken = urlParams.get('token'); // Backwards compatibility
        const newUser = urlParams.get('newUser');
        const error = urlParams.get('error');

        if (accessToken && refreshToken) {
            setAuthTokens(accessToken, refreshToken);
            if (newUser === 'true') {
                setIsNewUser(true);
            }
            // Clean URL
            window.history.replaceState({}, document.title, window.location.pathname);
        } else if (legacyToken) {
            // Handle legacy single token format
            setAuthTokens(legacyToken, '');
            if (newUser === 'true') {
                setIsNewUser(true);
            }
            window.history.replaceState({}, document.title, window.location.pathname);
        }

        if (error) {
            console.error('Auth error:', error);
            window.history.replaceState({}, document.title, window.location.pathname);
        }

        // Try to fetch current user
        const fetchUser = async () => {
            const existingToken = getAccessToken();
            if (!existingToken) {
                setIsLoading(false);
                return;
            }

            try {
                const userData = await apiFetch<User>('/auth/me');
                setUser(userData);
            } catch (err) {
                console.error('Failed to fetch user:', err);
                clearAuthTokens();
            } finally {
                setIsLoading(false);
            }
        };

        fetchUser();
    }, []);

    const login = () => {
        window.location.href = getGoogleAuthUrl();
    };

    const logout = async () => {
        await logoutApi();
        setUser(null);
        setIsNewUser(false);
    };

    const setUsername = async (username: string): Promise<{ success: boolean; error?: string }> => {
        try {
            setError(null);
            await apiFetch('/auth/set-username', {
                method: 'POST',
                body: JSON.stringify({ username }),
            });
            // Refresh user data
            const userData = await apiFetch<User>('/auth/me');
            setUser(userData);
            setIsNewUser(false);
            return { success: true };
        } catch (err) {
            const errorMessage = getErrorMessage(err);
            console.error('Failed to set username:', err);
            setError(errorMessage);
            return { success: false, error: errorMessage };
        }
    };

    const clearNewUser = () => {
        setIsNewUser(false);
    };

    const clearError = () => {
        setError(null);
    };

    return (
        <AuthContext.Provider
            value={{
                user,
                isLoading,
                isAuthenticated: !!user,
                isNewUser,
                error,
                login,
                logout,
                setUsername,
                clearNewUser,
                clearError,
            }}
        >
            {children}
        </AuthContext.Provider>
    );
}

export function useAuth() {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
}
