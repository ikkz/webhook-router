
import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useAuth } from '../hooks/use-auth';
import { Loader2 } from 'lucide-react';
import { checkAuth } from '@webhook-router/api-client';

export function LoginPage() {
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState('');

    const { login } = useAuth();
    const navigate = useNavigate();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setIsLoading(true);
        setError('');

        try {
            const token = btoa(`${username}:${password}`);
            const basicAuth = `Basic ${token}`;

            // Verify credentials with backend
            const { error } = await checkAuth({
                headers: {
                    Authorization: basicAuth
                }
            });

            if (error) {
                setError('Invalid username or password');
                return;
            }

            login(basicAuth);
            navigate({ to: '/' });
        } catch (err) {
            console.error('Login error', err);
            setError('Failed to login. Please check your network connection.');
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="flex min-h-screen items-center justify-center bg-gray-50/50">
            <div className="w-full max-w-sm space-y-6 p-6 bg-white rounded-lg border shadow-sm">
                <div className="space-y-2 text-center">
                    <h1 className="text-2xl font-bold">Login</h1>
                    <p className="text-muted-foreground">Enter your credentials to access the console</p>
                </div>

                {error && <div className="p-3 text-sm text-destructive bg-destructive/10 rounded-md">{error}</div>}

                <form onSubmit={handleSubmit} className="space-y-4">
                    <div className="space-y-2">
                        <label className="text-sm font-medium">Username</label>
                        <input
                            className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
                            value={username}
                            onChange={(e) => setUsername(e.target.value)}
                            required
                        />
                    </div>
                    <div className="space-y-2">
                        <label className="text-sm font-medium">Password</label>
                        <input
                            type="password"
                            className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            required
                        />
                    </div>
                    <button
                        type="submit"
                        disabled={isLoading}
                        className="inline-flex w-full items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2"
                    >
                        {isLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                        Login
                    </button>
                </form>
            </div>
        </div>
    );
}
