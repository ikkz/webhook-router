import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { listEndpoints, createEndpoint, CreateEndpointRequest } from '@webhook-router/api-client';
import { useState } from 'react';
import { Plus, Loader2 } from 'lucide-react';
import { Link } from '@tanstack/react-router';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

export function EndpointsPage() {
    const queryClient = useQueryClient();
    const [isCreating, setIsCreating] = useState(false);

    const { data: endpoints, isLoading, error } = useQuery({
        queryKey: ['endpoints'],
        queryFn: async () => {
            const res = await listEndpoints();
            return res.data;
        }
    });

    const createMutation = useMutation({
        mutationFn: async (data: CreateEndpointRequest) => {
            const res = await createEndpoint({ body: data });
            return res.data;
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['endpoints'] });
            setIsCreating(false);
        },
    });

    if (isLoading) return <div className="flex justify-center p-8"><Loader2 className="animate-spin" /></div>;
    if (error) return <div className="p-4 text-destructive">Error loading endpoints</div>;

    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center">
                <h2 className="text-2xl font-bold tracking-tight">Endpoints</h2>
                <Button onClick={() => setIsCreating(true)}>
                    <Plus className="w-4 h-4" />
                    New Endpoint
                </Button>
            </div>

            {isCreating && (
                <CreateEndpointForm
                    onSubmit={(data) => createMutation.mutate(data)}
                    onCancel={() => setIsCreating(false)}
                    isLoading={createMutation.isPending}
                />
            )}

            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {endpoints?.map((endpoint) => (
                    <Link
                        key={endpoint.id}
                        to="/endpoints/$endpointId"
                        params={{ endpointId: endpoint.id }}
                        className="block hover:border-primary transition-colors cursor-pointer"
                    >
                        <Card>
                            <CardHeader>
                                <CardTitle>{endpoint.name}</CardTitle>
                                <CardDescription>{endpoint.id}</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="text-xs text-muted-foreground mt-2">Created: {new Date(endpoint.created_at * 1000).toLocaleString()}</div>
                            </CardContent>
                        </Card>
                    </Link>
                ))}
                {endpoints?.length === 0 && (
                    <p className="col-span-full text-center text-muted-foreground py-10">No endpoints found. Create one to get started.</p>
                )}
            </div>
        </div>
    );
}

function CreateEndpointForm({ onSubmit, onCancel, isLoading }: { onSubmit: (data: CreateEndpointRequest) => void, onCancel: () => void, isLoading: boolean }) {
    const [name, setName] = useState('');

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        onSubmit({ name });
    };

    return (
        <Card className="mb-6">
            <CardHeader>
                <CardTitle>Create New Endpoint</CardTitle>
            </CardHeader>
            <CardContent>
                <form onSubmit={handleSubmit} className="space-y-4">
                    <div className="space-y-2">
                        <Label htmlFor="endpoint-name">Name</Label>
                        <Input
                            id="endpoint-name"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            placeholder="e.g., Production Webhooks"
                            required
                        />
                    </div>
                    <div className="flex justify-end gap-2">
                        <Button
                            type="button"
                            variant="outline"
                            onClick={onCancel}
                        >
                            Cancel
                        </Button>
                        <Button
                            type="submit"
                            disabled={isLoading}
                        >
                            {isLoading ? <Loader2 className="w-4 h-4 animate-spin mr-2" /> : null}
                            Create
                        </Button>
                    </div>
                </form>
            </CardContent>
        </Card>
    )
}
