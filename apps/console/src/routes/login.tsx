
import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useAuth } from '../hooks/use-auth';
import { Loader2 } from 'lucide-react';
import { checkAuth } from '@webhook-router/api-client';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

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

            await login(basicAuth);

            // Wait for the auth state to propagate to the router context
            // This ensures the beforeLoad check in protected routes sees the updated auth state
            await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));

            // Use replace to avoid adding login page to history
            navigate({ to: '/', replace: true });
        } catch (err) {
            console.error('Login error', err);
            setError('Failed to login. Please check your network connection.');
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="flex min-h-screen items-center justify-center bg-gray-50/50">
            <Card className="w-full max-w-sm">
                <CardHeader className="text-center">
                    <CardTitle className="text-2xl">Login</CardTitle>
                    <CardDescription>Enter your credentials to access the console</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    {error && <div className="p-3 text-sm text-destructive bg-destructive/10 rounded-md">{error}</div>}

                    <form onSubmit={handleSubmit} className="space-y-4">
                        <div className="space-y-2">
                            <Label htmlFor="username">Username</Label>
                            <Input
                                id="username"
                                value={username}
                                onChange={(e) => setUsername(e.target.value)}
                                required
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="password">Password</Label>
                            <Input
                                id="password"
                                type="password"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                required
                            />
                        </div>
                        <Button
                            type="submit"
                            disabled={isLoading}
                            className="w-full"
                        >
                            {isLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                            Login
                        </Button>
                    </form>
                </CardContent>
            </Card>
        </div>
    );
}
