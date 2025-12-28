
import { useParams } from '@tanstack/react-router';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { listTargets, createTarget, deleteTarget, getEndpoint, CreateTargetRequest } from '@webhook-router/api-client';
import { useState } from 'react';
import { Plus, Loader2, Trash2, Globe, MessageSquare, ArrowLeft } from 'lucide-react';
import { Link } from '@tanstack/react-router';

export function EndpointDetailsPage() {
    const { endpointId } = useParams({ strict: false });
    const queryClient = useQueryClient();
    const [isCreating, setIsCreating] = useState(false);

    // Fetch Endpoint Details
    const { data: endpoint, isLoading: isEndpointLoading } = useQuery({
        queryKey: ['endpoints', endpointId],
        queryFn: async () => {
            // @ts-expect-error generated client types might be slightly off or strict
            const res = await getEndpoint({ path: { id: endpointId } });
            return res.data;
        }
    });

    // Fetch Targets
    const { data: targets, isLoading: isTargetsLoading, error } = useQuery({
        queryKey: ['targets', endpointId], // Scope query key by endpointId
        queryFn: async () => {
            // @ts-expect-error generated client types might be slightly off or strict
            const res = await listTargets({ path: { id: endpointId } });
            return res.data;
        }
    });

    const createMutation = useMutation({
        mutationFn: async (data: CreateTargetRequest) => {
            // @ts-expect-error generated client types
            const res = await createTarget({ path: { id: endpointId }, body: data });
            return res.data;
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['targets', endpointId] });
            setIsCreating(false);
        },
    });

    const deleteMutation = useMutation({
        mutationFn: async (targetId: string) => {
            // @ts-expect-error generated client types
            await deleteTarget({ path: { id: endpointId, target_id: targetId } });
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['targets', endpointId] });
        }
    });

    if (isEndpointLoading || isTargetsLoading) return <div className="flex justify-center p-8"><Loader2 className="animate-spin" /></div>;
    if (error) return <div className="p-4 text-destructive">Error loading details</div>;

    if (!endpoint) return <div className="p-4">Endpoint not found</div>;

    return (
        <div className="space-y-6">
            <div>
                <Link to="/endpoints" className="text-sm text-muted-foreground hover:underline flex items-center mb-2">
                    <ArrowLeft className="w-4 h-4 mr-1" /> Back to Endpoints
                </Link>
                <h2 className="text-2xl font-bold tracking-tight">{endpoint.name}</h2>
                <div className="text-muted-foreground text-sm flex gap-4 mt-2">
                    <span>ID: {endpoint.id}</span>
                    <span>Created: {new Date(endpoint.created_at * 1000).toLocaleString()}</span>
                </div>
            </div>

            <div className="border-t pt-6">
                <div className="flex justify-between items-center mb-4">
                    <h3 className="text-xl font-semibold tracking-tight">Targets</h3>
                    <button
                        onClick={() => setIsCreating(true)}
                        className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2"
                    >
                        <Plus className="w-4 h-4" />
                        Add Target
                    </button>
                </div>

                {isCreating && (
                    <CreateTargetForm
                        onSubmit={(data) => createMutation.mutate(data)}
                        onCancel={() => setIsCreating(false)}
                        isLoading={createMutation.isPending}
                    />
                )}

                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {targets?.map((target) => (
                        <div key={target.id} className="rounded-lg border bg-card text-card-foreground shadow-sm">
                            <div className="flex flex-col space-y-1.5 p-6">
                                <div className="flex justify-between items-start">
                                    <h3 className="text-xl font-semibold leading-none tracking-tight flex items-center gap-2">
                                        {target.kind === 'slack' ? <MessageSquare className="w-4 h-4" /> : <Globe className="w-4 h-4" />}
                                        {target.name}
                                    </h3>
                                    <button
                                        onClick={() => {
                                            if (confirm('Are you sure you want to delete this target?')) {
                                                deleteMutation.mutate(target.id);
                                            }
                                        }}
                                        className="text-destructive hover:bg-destructive/10 p-1 rounded"
                                    >
                                        <Trash2 className="w-4 h-4" />
                                    </button>
                                </div>
                                <p className="text-sm text-muted-foreground truncate" title={target.url}>{target.url}</p>
                            </div>
                            <div className="p-6 pt-0">
                                <div className="text-xs text-muted-foreground">Kind: {target.kind}</div>
                                <div className="text-xs text-muted-foreground mt-1">Created: {new Date(target.created_at * 1000).toLocaleString()}</div>
                            </div>
                        </div>
                    ))}
                    {targets?.length === 0 && (
                        <p className="col-span-full text-center text-muted-foreground py-10 border rounded-lg border-dashed">No targets configured for this endpoint.</p>
                    )}
                </div>
            </div>
        </div>
    );
}

function CreateTargetForm({ onSubmit, onCancel, isLoading }: { onSubmit: (data: CreateTargetRequest) => void, onCancel: () => void, isLoading: boolean }) {
    const [name, setName] = useState('');
    const [url, setUrl] = useState('');
    const [kind, setKind] = useState('http');

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        onSubmit({ name, url, kind, headers: {} });
    };

    return (
        <div className="rounded-lg border bg-card text-card-foreground shadow-sm p-6 mb-6">
            <h3 className="font-semibold mb-4">Add Target to Endpoint</h3>
            <form onSubmit={handleSubmit} className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                        <label className="text-sm font-medium">Name</label>
                        <input
                            className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            placeholder="My Webhook"
                            required
                        />
                    </div>
                    <div className="space-y-2">
                        <label className="text-sm font-medium">Kind</label>
                        <select
                            className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                            value={kind}
                            onChange={(e) => setKind(e.target.value)}
                        >
                            <option value="http">HTTP</option>
                            <option value="slack">Slack</option>
                            <option value="dingtalk">DingTalk</option>
                            <option value="lark">Lark</option>
                        </select>
                    </div>
                </div>
                <div className="space-y-2">
                    <label className="text-sm font-medium">URL</label>
                    <input
                        className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                        value={url}
                        onChange={(e) => setUrl(e.target.value)}
                        placeholder="https://..."
                        required
                    />
                </div>
                <div className="flex justify-end gap-2">
                    <button
                        type="button"
                        onClick={onCancel}
                        className="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium border border-input bg-background hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2"
                    >
                        Cancel
                    </button>
                    <button
                        type="submit"
                        disabled={isLoading}
                        className="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2"
                    >
                        {isLoading ? <Loader2 className="w-4 h-4 animate-spin mr-2" /> : null}
                        Create
                    </button>
                </div>
            </form>
        </div>
    )
}
