import { createContext, useContext, useState, useEffect } from 'react';
import type { ReactNode } from 'react';
import type { User } from '../types/auth';
import { apiFetch, setAuthToken, clearAuthToken, getAuthToken, getGoogleAuthUrl } from '../utils/api';

interface AuthContextType {
    user: User | null;
    isLoading: boolean;
    isAuthenticated: boolean;
    isNewUser: boolean;
    login: () => void;
    logout: () => void;
    setUsername: (username: string) => Promise<boolean>;
    clearNewUser: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
    const [user, setUser] = useState<User | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isNewUser, setIsNewUser] = useState(false);

    useEffect(() => {
        // Check for token in URL (OAuth callback)
        const urlParams = new URLSearchParams(window.location.search);
        const token = urlParams.get('token');
        const newUser = urlParams.get('newUser');
        const error = urlParams.get('error');

        if (token) {
            setAuthToken(token);
            if (newUser === 'true') {
                setIsNewUser(true);
            }
            // Clean URL
            window.history.replaceState({}, document.title, window.location.pathname);
        }

        if (error) {
            console.error('Auth error:', error);
            window.history.replaceState({}, document.title, window.location.pathname);
        }

        // Try to fetch current user
        const fetchUser = async () => {
            const existingToken = getAuthToken();
            if (!existingToken) {
                setIsLoading(false);
                return;
            }

            try {
                const userData = await apiFetch<User>('/auth/me');
                setUser(userData);
            } catch (err) {
                console.error('Failed to fetch user:', err);
                clearAuthToken();
            } finally {
                setIsLoading(false);
            }
        };

        fetchUser();
    }, []);

    const login = () => {
        window.location.href = getGoogleAuthUrl();
    };

    const logout = () => {
        clearAuthToken();
        setUser(null);
        setIsNewUser(false);
    };

    const setUsername = async (username: string): Promise<boolean> => {
        try {
            await apiFetch('/auth/set-username', {
                method: 'POST',
                body: JSON.stringify({ username }),
            });
            // Refresh user data
            const userData = await apiFetch<User>('/auth/me');
            setUser(userData);
            setIsNewUser(false);
            return true;
        } catch (err) {
            console.error('Failed to set username:', err);
            return false;
        }
    };

    const clearNewUser = () => {
        setIsNewUser(false);
    };

    return (
        <AuthContext.Provider
            value={{
                user,
                isLoading,
                isAuthenticated: !!user,
                isNewUser,
                login,
                logout,
                setUsername,
                clearNewUser,
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
