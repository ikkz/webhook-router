import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { listEndpoints, createEndpoint, CreateEndpointRequest } from '@webhook-router/api-client';
import { useState } from 'react';
import { Plus, Loader2 } from 'lucide-react';

import { Link } from '@tanstack/react-router';

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
                <button
                    onClick={() => setIsCreating(true)}
                    className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2"
                >
                    <Plus className="w-4 h-4" />
                    New Endpoint
                </button>
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
                        className="block rounded-lg border bg-card text-card-foreground shadow-sm hover:border-primary transition-colors cursor-pointer"
                    >
                        <div className="flex flex-col space-y-1.5 p-6">
                            <h3 className="text-2xl font-semibold leading-none tracking-tight">{endpoint.name}</h3>
                            <p className="text-sm text-muted-foreground">{endpoint.id}</p>
                        </div>
                        <div className="p-6 pt-0">
                            <div className="text-xs text-muted-foreground mt-2">Created: {new Date(endpoint.created_at * 1000).toLocaleString()}</div>
                        </div>
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
        <div className="rounded-lg border bg-card text-card-foreground shadow-sm p-6 mb-6">
            <h3 className="font-semibold mb-4">Create New Endpoint</h3>
            <form onSubmit={handleSubmit} className="space-y-4">
                <div className="space-y-2">
                    <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">Name</label>
                    <input
                        className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        placeholder="e.g., Production Webhooks"
                        required
                    />
                </div>
                <div className="flex justify-end gap-2">
                    <button
                        type="button"
                        onClick={onCancel}
                        className="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2"
                    >
                        Cancel
                    </button>
                    <button
                        type="submit"
                        disabled={isLoading}
                        className="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2"
                    >
                        {isLoading ? <Loader2 className="w-4 h-4 animate-spin mr-2" /> : null}
                        Create
                    </button>
                </div>
            </form>
        </div>
    )
}
