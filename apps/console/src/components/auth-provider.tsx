import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { checkAuth, client } from '@webhook-router/api-client';

const STORAGE_KEY = 'webhook_router_auth';

// Configure global client interceptor
client.interceptors.request.use((request) => {
    const token = localStorage.getItem(STORAGE_KEY);
    if (token) {
        request.headers.set('Authorization', token);
    }
    return request;
});

// Handle 403 Forbidden responses by clearing auth
client.interceptors.response.use((response) => {
    if (response.status === 403) {
        localStorage.removeItem(STORAGE_KEY);
        // Trigger a custom event to notify the app to logout
        window.dispatchEvent(new CustomEvent('auth:forbidden'));
    }
    return response;
});

interface AuthContextType {
    isAuthenticated: boolean;
    login: (token: string) => Promise<void>;
    logout: () => void;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
    const [isAuthenticated, setIsAuthenticated] = useState(() => {
        return !!localStorage.getItem(STORAGE_KEY);
    });

    const login = async (token: string) => {
        localStorage.setItem(STORAGE_KEY, token);
        setIsAuthenticated(true);
        // Wait for state update to complete
        await new Promise(resolve => setTimeout(resolve, 0));
    };

    const logout = () => {
        localStorage.removeItem(STORAGE_KEY);
        setIsAuthenticated(false);
    };

    useEffect(() => {
        if (!isAuthenticated) return;

        checkAuth().then(({ error }) => {
            if (error) {
                logout();
            }
        }).catch(() => {
            // Ignore network errors for optimistic auth
        });
    }, []);

    useEffect(() => {
        const handleForbidden = () => {
            logout();
        };

        window.addEventListener('auth:forbidden', handleForbidden);
        return () => window.removeEventListener('auth:forbidden', handleForbidden);
    }, []);

    return (
        <AuthContext.Provider value={{ isAuthenticated, login, logout }}>
            {children}
        </AuthContext.Provider>
    );
}

export function useAuth() {
    const context = useContext(AuthContext);
    if (!context) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
}
