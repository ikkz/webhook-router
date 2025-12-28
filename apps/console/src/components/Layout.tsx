
import { Link, Outlet } from '@tanstack/react-router';

export function Layout() {
    return (
        <div className="flex flex-col min-h-screen">
            <header className="p-4 border-b flex gap-4">
                <Link to="/" className="font-bold">
                    Console
                </Link>
            </header>
            <main className="flex-1 p-4">
                <Outlet />
            </main>
        </div>
    );
}
