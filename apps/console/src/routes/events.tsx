import { useQuery } from '@tanstack/react-query';
import { listEvents, listEndpoints, EventRecord } from '@webhook-router/api-client';
import { Loader2, FileText, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Link } from '@tanstack/react-router';
import { useState } from 'react';

export function EventsPage() {
    const [previewEvent, setPreviewEvent] = useState<EventRecord | null>(null);

    const { data: events, isLoading: isEventsLoading, error: eventsError } = useQuery({
        queryKey: ['events'],
        queryFn: async () => {
            const res = await listEvents();
            return res.data;
        },
        refetchInterval: 5000,
    });

    const { data: endpoints, isLoading: isEndpointsLoading } = useQuery({
        queryKey: ['endpoints'],
        queryFn: async () => {
            const res = await listEndpoints();
            return res.data;
        },
    });

    const isLoading = isEventsLoading || isEndpointsLoading;

    if (isLoading) return <div className="flex justify-center p-8"><Loader2 className="animate-spin" /></div>;
    if (eventsError) return <div className="p-4 text-destructive">Error loading events</div>;

    const endpointMap = new Map(endpoints?.map(e => [e.id, e.name]));

    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center">
                <h2 className="text-2xl font-bold tracking-tight">Events</h2>
            </div>

            <div className="rounded-md border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="w-[100px]">ID</TableHead>
                            <TableHead className="w-[150px]">Endpoint</TableHead>
                            <TableHead className="w-[100px]">Platform</TableHead>
                            <TableHead>Title</TableHead>
                            <TableHead className="w-[200px]">Time</TableHead>
                            <TableHead className="w-[80px]">Preview</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {events?.map((event) => (
                            <TableRow key={event.id}>
                                <TableCell className="font-mono text-xs">{event.id?.slice(0, 8)}</TableCell>
                                <TableCell>
                                    <Link
                                        to="/endpoints/$endpointId"
                                        params={{ endpointId: event.endpoint_id }}
                                        className="text-primary hover:underline font-medium"
                                    >
                                        {endpointMap.get(event.endpoint_id) || event.endpoint_id.slice(0, 8)}
                                    </Link>
                                </TableCell>
                                <TableCell className="capitalize">{event.platform}</TableCell>
                                <TableCell className="truncate max-w-[300px]">{event.title || '-'}</TableCell>
                                <TableCell className="text-muted-foreground">{new Date(event.created_at * 1000).toLocaleString()}</TableCell>
                                <TableCell>
                                    <Button
                                        variant="outline"
                                        size="icon-sm"
                                        onClick={() => setPreviewEvent(event)}
                                    >
                                        <FileText className="w-4 h-4" />
                                    </Button>
                                </TableCell>
                            </TableRow>
                        ))}
                        {events?.length === 0 && (
                            <TableRow>
                                <TableCell colSpan={6} className="text-center text-muted-foreground">No events found.</TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>

            {previewEvent && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
                    <div className="bg-background rounded-lg shadow-lg w-full max-w-2xl max-h-[80vh] flex flex-col">
                        <div className="flex items-center justify-between p-4 border-b">
                            <h3 className="text-lg font-semibold">Event Details</h3>
                            <Button variant="ghost" size="icon-sm" onClick={() => setPreviewEvent(null)}>
                                <X className="w-4 h-4" />
                            </Button>
                        </div>
                        <div className="p-4 overflow-y-auto space-y-4 flex-1">
                            <div>
                                <h4 className="text-sm font-medium mb-1">Markdown Content</h4>
                                <div className="bg-muted p-3 rounded-md text-sm whitespace-pre-wrap font-mono">
                                    {previewEvent.markdown || '(No content)'}
                                </div>
                            </div>
                            <div>
                                <h4 className="text-sm font-medium mb-1">Raw Data</h4>
                                <pre className="bg-muted p-3 rounded-md text-xs overflow-x-auto">
                                    {JSON.stringify(previewEvent.raw, null, 2)}
                                </pre>
                            </div>
                        </div>
                        <div className="p-4 border-t flex justify-end">
                            <Button onClick={() => setPreviewEvent(null)}>Close</Button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}

