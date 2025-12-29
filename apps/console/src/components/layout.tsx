import { Outlet } from '@tanstack/react-router';
import { Sidebar } from './sidebar';

export function Layout() {
    return (
        <div className="flex h-screen bg-background text-foreground overflow-hidden">
            <Sidebar />
            <div className="flex-1 flex flex-col overflow-hidden">
                <header className="h-14 border-b px-4 flex items-center justify-between flex-shrink-0">
                    <h1 className="font-semibold">Console</h1>
                </header>
                <main className="flex-1 p-6 overflow-y-auto">
                    <Outlet />
                </main>
            </div>
        </div>
    );
}
