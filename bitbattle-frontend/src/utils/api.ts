const API_BASE_URL = 'http://localhost:4000';

export async function apiFetch<T>(
    endpoint: string,
    options: RequestInit = {}
): Promise<T> {
    const token = localStorage.getItem('bitbattle_token');

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

    if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
    }

    return response.json();
}

export function getAuthToken(): string | null {
    return localStorage.getItem('bitbattle_token');
}

export function setAuthToken(token: string): void {
    localStorage.setItem('bitbattle_token', token);
}

export function clearAuthToken(): void {
    localStorage.removeItem('bitbattle_token');
}

export function getGoogleAuthUrl(): string {
    return `${API_BASE_URL}/auth/google`;
}
