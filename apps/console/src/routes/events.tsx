import { useQuery } from '@tanstack/react-query';
import { listEvents } from '@webhook-router/api-client';
import { Loader2, FileText } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';


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
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>ID</TableHead>
                            <TableHead>Platform</TableHead>
                            <TableHead>Title</TableHead>
                            <TableHead>Time</TableHead>
                            <TableHead>Preview</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {events?.map((event) => (
                            <TableRow key={event.id}>
                                <TableCell className="font-mono text-xs">{event.id?.slice(0, 8)}</TableCell>
                                <TableCell>{event.platform}</TableCell>
                                <TableCell>{event.title || '-'}</TableCell>
                                <TableCell className="text-muted-foreground">{new Date(event.created_at * 1000).toLocaleString()}</TableCell>
                                <TableCell>
                                    <Button variant="outline" size="icon-sm">
                                        <FileText className="w-4 h-4" />
                                    </Button>
                                </TableCell>
                            </TableRow>
                        ))}
                        {events?.length === 0 && (
                            <TableRow>
                                <TableCell colSpan={5} className="text-center text-muted-foreground">No events found.</TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>
        </div>
    );
}
