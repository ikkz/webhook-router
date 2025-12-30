import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { listEndpoints, createEndpoint, updateEndpoint, deleteEndpoint, CreateEndpointRequest } from '@webhook-router/api-client';
import { useState } from 'react';
import { Plus, Loader2, Pencil, Trash2, X, Check } from 'lucide-react';
import { useNavigate } from '@tanstack/react-router';
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
                    <EndpointCard key={endpoint.id} endpoint={endpoint} />
                ))}
                {endpoints?.length === 0 && (
                    <p className="col-span-full text-center text-muted-foreground py-10">No endpoints found. Create one to get started.</p>
                )}
            </div>
        </div>
    );
}

function EndpointCard({ endpoint }: { endpoint: any }) {
    const queryClient = useQueryClient();
    const navigate = useNavigate();
    const [isEditing, setIsEditing] = useState(false);
    const [editName, setEditName] = useState(endpoint.name);

    const updateMutation = useMutation({
        mutationFn: async (name: string) => {
            await updateEndpoint({ path: { id: endpoint.id }, body: { name } });
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['endpoints'] });
            setIsEditing(false);
        },
    });

    const deleteMutation = useMutation({
        mutationFn: async () => {
            await deleteEndpoint({ path: { id: endpoint.id } });
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['endpoints'] });
        },
    });

    const handleSave = (e: React.FormEvent) => {
        e.preventDefault();
        e.stopPropagation();
        if (editName.trim() && editName !== endpoint.name) {
            updateMutation.mutate(editName);
        } else {
            setIsEditing(false);
        }
    };

    const handleDelete = (e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        if (window.confirm('Are you sure you want to delete this endpoint?')) {
            deleteMutation.mutate();
        }
    };

    const handleCardClick = (e: React.MouseEvent) => {
        // Only navigate if not editing and not clicking on controls
        if (!isEditing) {
            navigate({ to: '/endpoints/$endpointId', params: { endpointId: endpoint.id } });
        }
    };

    return (
        <Card className="hover:border-primary transition-colors cursor-pointer group relative" onClick={handleCardClick}>
            <CardHeader className="pb-2">
                <div className="flex justify-between items-start gap-2">
                    {isEditing ? (
                        <form onSubmit={handleSave} className="flex-1 flex items-center gap-2" onClick={e => e.stopPropagation()}>
                            <Input
                                value={editName}
                                onChange={(e) => setEditName(e.target.value)}
                                className="h-8 text-sm"
                                autoFocus
                            />
                            <Button size="icon-sm" type="submit" disabled={updateMutation.isPending} variant="ghost" className="h-8 w-8">
                                <Check className="w-4 h-4 text-green-500" />
                            </Button>
                            <Button size="icon-sm" type="button" onClick={() => setIsEditing(false)} variant="ghost" className="h-8 w-8">
                                <X className="w-4 h-4 text-muted-foreground" />
                            </Button>
                        </form>
                    ) : (
                        <>
                            <CardTitle className="leading-none pt-1">{endpoint.name}</CardTitle>
                            <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                <Button
                                    size="icon-sm"
                                    variant="ghost"
                                    className="h-8 w-8 text-muted-foreground hover:text-primary"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        setIsEditing(true);
                                        setEditName(endpoint.name);
                                    }}
                                >
                                    <Pencil className="w-3.5 h-3.5" />
                                </Button>
                                <Button
                                    size="icon-sm"
                                    variant="ghost"
                                    className="h-8 w-8 text-muted-foreground hover:text-destructive"
                                    onClick={handleDelete}
                                >
                                    <Trash2 className="w-3.5 h-3.5" />
                                </Button>
                            </div>
                        </>
                    )}
                </div>
                <CardDescription className="text-xs font-mono mt-1">{endpoint.id}</CardDescription>
            </CardHeader>
            <CardContent>
                <div className="text-xs text-muted-foreground">Created: {new Date(endpoint.created_at * 1000).toLocaleString()}</div>
            </CardContent>
        </Card>
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
