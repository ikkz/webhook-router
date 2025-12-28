import { useQuery } from '@tanstack/react-query';
import { listEvents } from '@webhook-router/api-client';
import { Loader2, FileJson, FileText } from 'lucide-react';


export function EventsPage() {
    const { data: events, isLoading, error } = useQuery({
        queryKey: ['events'],
        queryFn: async () => {
            const res = await listEvents();
            return res.data;
        },
        refetchInterval: 5000,
    });

    if (isLoading) return <div className="flex justify-center p-8"><Loader2 className="animate-spin" /></div>;
    if (error) return <div className="p-4 text-destructive">Error loading events</div>;

    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center">
                <h2 className="text-2xl font-bold tracking-tight">Events</h2>
            </div>

            <div className="rounded-md border">
                <div className="relative w-full overflow-auto">
                    <table className="w-full caption-bottom text-sm">
                        <thead className="[&_tr]:border-b">
                            <tr className="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                <th className="h-12 px-4 text-left align-middle font-medium text-muted-foreground">ID</th>
                                <th className="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Platform</th>
                                <th className="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Title</th>
                                <th className="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Time</th>
                                <th className="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Preview</th>
                            </tr>
                        </thead>
                        <tbody className="[&_tr:last-child]:border-0">
                            {events?.map((event) => (
                                <tr key={event.id} className="border-b transition-colors hover:bg-muted/50">
                                    <td className="p-4 align-middle font-mono text-xs">{event.id?.slice(0, 8)}</td>
                                    <td className="p-4 align-middle">{event.platform}</td>
                                    <td className="p-4 align-middle">{event.title || '-'}</td>
                                    <td className="p-4 align-middle text-muted-foreground">{new Date(event.created_at * 1000).toLocaleString()}</td>
                                    <td className="p-4 align-middle">
                                        <div className="flex gap-2">
                                            <button className="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-8 w-8">
                                                <FileText className="w-4 h-4" />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                            {events?.length === 0 && (
                                <tr>
                                    <td colSpan={5} className="p-4 text-center text-muted-foreground">No events found.</td>
                                </tr>
                            )}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
}
