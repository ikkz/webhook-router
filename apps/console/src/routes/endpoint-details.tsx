
import { useParams } from '@tanstack/react-router';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { listTargets, createTarget, deleteTarget, getEndpoint, updateEndpoint, CreateTargetRequest, UpdateEndpointRequest, testSend } from '@webhook-router/api-client';
import { useState, useRef, useEffect } from 'react';
import { Plus, Loader2, Trash2, Globe, MessageSquare, ArrowLeft, Copy, Check, Send, CheckCircle2, XCircle } from 'lucide-react';
import { Link } from '@tanstack/react-router';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { EditorView, basicSetup } from 'codemirror';
import { markdown } from '@codemirror/lang-markdown';

export function EndpointDetailsPage() {
    const { endpointId } = useParams({ strict: false }) as { endpointId: string };
    const queryClient = useQueryClient();
    const [isCreating, setIsCreating] = useState(false);

    // Fetch Endpoint Details
    const { data: endpoint, isLoading: isEndpointLoading } = useQuery({
        queryKey: ['endpoints', endpointId],
        queryFn: async () => {
            const res = await getEndpoint({ path: { id: endpointId } });
            return res.data;
        }
    });

    // Fetch Targets
    const { data: targets, isLoading: isTargetsLoading, error } = useQuery({
        queryKey: ['targets', endpointId], // Scope query key by endpointId
        queryFn: async () => {
            const res = await listTargets({ path: { id: endpointId } });
            return res.data;
        }
    });

    const createMutation = useMutation({
        mutationFn: async (data: CreateTargetRequest) => {
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

            <div className="border-t pt-6 space-y-6">
                <ConfigurationSection endpoint={endpoint} endpointId={endpointId} />
                <TestSendSection endpointId={endpointId} />
                <WebhookUrls endpointId={endpoint.id} />

                <div className="flex justify-between items-center mb-4">
                    <h3 className="text-xl font-semibold tracking-tight">Targets</h3>
                    <Button onClick={() => setIsCreating(true)}>
                        <Plus className="w-4 h-4" />
                        Add Target
                    </Button>
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
                        <Card key={target.id}>
                            <CardHeader>
                                <div className="flex justify-between items-start">
                                    <CardTitle className="flex items-center gap-2">
                                        {target.kind === 'slack' ? <MessageSquare className="w-4 h-4" /> : <Globe className="w-4 h-4" />}
                                        {target.name}
                                    </CardTitle>
                                    <Button
                                        variant="ghost"
                                        size="icon-sm"
                                        onClick={() => {
                                            if (window.confirm('Are you sure you want to delete this target?')) {
                                                deleteMutation.mutate(target.id);
                                            }
                                        }}
                                        className="text-destructive hover:bg-destructive/10"
                                    >
                                        <Trash2 className="w-4 h-4" />
                                    </Button>
                                </div>
                                <CardDescription className="truncate" title={target.url}>{target.url}</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="text-xs text-muted-foreground">Kind: {target.kind}</div>
                                <div className="text-xs text-muted-foreground mt-1">Created: {new Date(target.created_at * 1000).toLocaleString()}</div>
                            </CardContent>
                        </Card>
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
        <Card className="mb-6">
            <CardHeader>
                <CardTitle>Add Target to Endpoint</CardTitle>
            </CardHeader>
            <CardContent>
                <form onSubmit={handleSubmit} className="space-y-4">
                    <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                            <Label htmlFor="target-name">Name</Label>
                            <Input
                                id="target-name"
                                value={name}
                                onChange={(e) => setName(e.target.value)}
                                placeholder="My Webhook"
                                required
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="target-kind">Kind</Label>
                            <Select value={kind} onValueChange={setKind}>
                                <SelectTrigger id="target-kind">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="http">HTTP</SelectItem>
                                    <SelectItem value="slack">Slack</SelectItem>
                                    <SelectItem value="dingtalk">DingTalk</SelectItem>
                                    <SelectItem value="lark">Lark</SelectItem>
                                    <SelectItem value="wecom">WeCom</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="target-url">URL</Label>
                        <Input
                            id="target-url"
                            value={url}
                            onChange={(e) => setUrl(e.target.value)}
                            placeholder="https://..."
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

function WebhookUrls({ endpointId }: { endpointId: string }) {
    const [copiedUrl, setCopiedUrl] = useState<string | null>(null);

    // Get the base URL from the current window location
    const baseUrl = window.location.origin;

    const platforms = [
        { name: 'Slack', value: 'slack' },
        { name: 'Lark/Feishu', value: 'lark' },
        { name: 'DingTalk', value: 'dingtalk' },
        { name: 'WeCom', value: 'wecom' },
        { name: 'HTTP/Custom', value: 'http' },
    ];

    const copyToClipboard = async (platform: string) => {
        const url = `${baseUrl}/ingress/${endpointId}/${platform}`;
        try {
            await navigator.clipboard.writeText(url);
            setCopiedUrl(url);
            setTimeout(() => setCopiedUrl(null), 2000);
        } catch (err) {
            console.error('Failed to copy:', err);
        }
    };

    return (
        <Card className="mb-6">
            <CardHeader>
                <CardTitle className="text-lg">Webhook Ingress URLs</CardTitle>
                <CardDescription>Copy these URLs to configure webhooks on different platforms</CardDescription>
            </CardHeader>
            <CardContent>
                <div className="space-y-3">
                    {platforms.map(({ name, value }) => {
                        const url = `${baseUrl}/ingress/${endpointId}/${value}`;
                        const isCopied = copiedUrl === url;

                        return (
                            <div key={value} className="flex items-end gap-2 group">
                                <div className="flex-1 min-w-0">
                                    <div className="text-xs font-medium text-muted-foreground mb-1">{name}</div>
                                    <code className="text-xs bg-muted px-3 py-2 rounded block truncate">
                                        {url}
                                    </code>
                                </div>
                                <Button
                                    variant="outline"
                                    size="icon-sm"
                                    onClick={() => copyToClipboard(value)}
                                    className="flex-shrink-0"
                                    title={`Copy ${name} URL`}
                                >
                                    {isCopied ? (
                                        <Check className="w-4 h-4 text-green-500" />
                                    ) : (
                                        <Copy className="w-4 h-4" />
                                    )}
                                </Button>
                            </div>
                        );
                    })}
                </div>
            </CardContent>
        </Card>
    );
}
function ConfigurationSection({ endpoint, endpointId }: { endpoint: any, endpointId: string }) {
    const queryClient = useQueryClient();
    const [banner, setBanner] = useState(endpoint.banner || '');
    const [footer, setFooter] = useState(endpoint.footer || '');
    const [hasChanges, setHasChanges] = useState(false);

    const bannerEditorRef = useRef<HTMLDivElement>(null);
    const footerEditorRef = useRef<HTMLDivElement>(null);
    const bannerViewRef = useRef<EditorView | null>(null);
    const footerViewRef = useRef<EditorView | null>(null);

    // Initialize CodeMirror editors
    useEffect(() => {
        if (bannerEditorRef.current && !bannerViewRef.current) {
            bannerViewRef.current = new EditorView({
                doc: banner,
                extensions: [
                    basicSetup,
                    markdown(),
                    EditorView.updateListener.of((update) => {
                        if (update.docChanged) {
                            const newValue = update.state.doc.toString();
                            setBanner(newValue);
                            setHasChanges(newValue !== (endpoint.banner || '') || footer !== (endpoint.footer || ''));
                        }
                    }),
                ],
                parent: bannerEditorRef.current,
            });
        }

        if (footerEditorRef.current && !footerViewRef.current) {
            footerViewRef.current = new EditorView({
                doc: footer,
                extensions: [
                    basicSetup,
                    markdown(),
                    EditorView.updateListener.of((update) => {
                        if (update.docChanged) {
                            const newValue = update.state.doc.toString();
                            setFooter(newValue);
                            setHasChanges(banner !== (endpoint.banner || '') || newValue !== (endpoint.footer || ''));
                        }
                    }),
                ],
                parent: footerEditorRef.current,
            });
        }

        return () => {
            if (bannerViewRef.current) {
                bannerViewRef.current.destroy();
                bannerViewRef.current = null;
            }
            if (footerViewRef.current) {
                footerViewRef.current.destroy();
                footerViewRef.current = null;
            }
        };
    }, []);

    const updateMutation = useMutation({
        mutationFn: async (data: UpdateEndpointRequest) => {
            const res = await updateEndpoint({ path: { id: endpointId }, body: data });
            return res.data;
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['endpoints', endpointId] });
            setHasChanges(false);
        },
    });

    const handleSave = () => {
        updateMutation.mutate({ banner, footer });
    };

    return (
        <Card>
            <CardHeader>
                <CardTitle>Markdown Configuration</CardTitle>
                <CardDescription>
                    Configure banner and footer text that will be added to all incoming events
                </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                <div className="space-y-2">
                    <Label>Banner (prepended to incoming messages)</Label>
                    <div ref={bannerEditorRef} className="border rounded min-h-[100px]" />
                </div>
                <div className="space-y-2">
                    <Label>Footer (appended to incoming messages)</Label>
                    <div ref={footerEditorRef} className="border rounded min-h-[100px]" />
                </div>
                <Button
                    onClick={handleSave}
                    disabled={!hasChanges || updateMutation.isPending}
                >
                    {updateMutation.isPending ? <Loader2 className="w-4 h-4 animate-spin mr-2" /> : null}
                    Save Configuration
                </Button>
                {updateMutation.isSuccess && (
                    <p className="text-sm text-green-600">Configuration saved successfully!</p>
                )}
            </CardContent>
        </Card>
    );
}

function TestSendSection({ endpointId }: { endpointId: string }) {
    const [testMarkdown, setTestMarkdown] = useState('# Test Message\n\nThis is a test message to verify your endpoint configuration.');
    const [deliveryResults, setDeliveryResults] = useState<any>(null);

    const editorRef = useRef<HTMLDivElement>(null);
    const viewRef = useRef<EditorView | null>(null);

    useEffect(() => {
        if (editorRef.current && !viewRef.current) {
            viewRef.current = new EditorView({
                doc: testMarkdown,
                extensions: [
                    basicSetup,
                    markdown(),
                    EditorView.updateListener.of((update) => {
                        if (update.docChanged) {
                            setTestMarkdown(update.state.doc.toString());
                        }
                    }),
                ],
                parent: editorRef.current,
            });
        }

        return () => {
            if (viewRef.current) {
                viewRef.current.destroy();
                viewRef.current = null;
            }
        };
    }, []);

    const testSendMutation = useMutation({
        mutationFn: async (markdown: string) => {
            const res = await testSend({
                path: { id: endpointId },
                body: { markdown },
            });
            return res.data;
        },
        onSuccess: (data) => {
            setDeliveryResults(data);
        },
    });

    const handleTestSend = () => {
        setDeliveryResults(null);
        testSendMutation.mutate(testMarkdown);
    };

    return (
        <Card>
            <CardHeader>
                <CardTitle>Test Send</CardTitle>
                <CardDescription>
                    Send a test message to verify your configuration (banner/footer will be automatically added)
                </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                <div className="space-y-2">
                    <Label>Test Markdown Content</Label>
                    <div ref={editorRef} className="border rounded min-h-[150px]" />
                </div>
                <Button
                    onClick={handleTestSend}
                    disabled={testSendMutation.isPending}
                >
                    {testSendMutation.isPending ? <Loader2 className="w-4 h-4 animate-spin mr-2" /> : <Send className="w-4 h-4 mr-2" />}
                    Send Test
                </Button>

                {deliveryResults && (
                    <div className="mt-4 p-4 border rounded bg-muted/50">
                        <h4 className="font-semibold mb-2">Delivery Results:</h4>
                        <p className="text-sm text-muted-foreground mb-2">Event ID: {deliveryResults.event_id}</p>
                        <div className="space-y-2">
                            {deliveryResults.deliveries?.map((delivery: any, idx: number) => (
                                <div key={idx} className="flex items-center gap-2 text-sm">
                                    {delivery.status === 'sent' ? (
                                        <CheckCircle2 className="w-4 h-4 text-green-500" />
                                    ) : (
                                        <XCircle className="w-4 h-4 text-red-500" />
                                    )}
                                    <span>Target {delivery.target_id}: {delivery.status}</span>
                                    {delivery.response_code && <span className="text-muted-foreground">({delivery.response_code})</span>}
                                    {delivery.error && <span className="text-destructive">- {delivery.error}</span>}
                                </div>
                            ))}
                            {(!deliveryResults.deliveries || deliveryResults.deliveries.length === 0) && (
                                <p className="text-sm text-muted-foreground">No targets configured for this endpoint</p>
                            )}
                        </div>
                    </div>
                )}

                {testSendMutation.isError && (
                    <div className="mt-4 p-4 border border-destructive rounded bg-destructive/10">
                        <p className="text-sm text-destructive">Failed to send test message</p>
                    </div>
                )}
            </CardContent>
        </Card>
    );
}
