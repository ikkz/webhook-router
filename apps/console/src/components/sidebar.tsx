
import { Link } from '@tanstack/react-router';
import { Activity, Radio } from 'lucide-react';

export function Sidebar() {
    const navItems = [
        { to: '/endpoints', label: 'Endpoints', icon: Radio },
        { to: '/events', label: 'Events', icon: Activity },
    ];

    return (
        <aside className="w-64 border-r bg-card h-full flex flex-col">
            <div className="p-4 font-bold text-xl border-b flex items-center gap-2">
                <Activity className="w-6 h-6 text-primary" />
                Webhook Router
            </div>
            <nav className="p-2 space-y-1">
                {navItems.map((item) => (
                    <Link
                        key={item.to}
                        to={item.to}
                        className="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-muted/50 transition-colors [&.active]:bg-muted [&.active]:font-medium"
                    >
                        <item.icon className="w-4 h-4" />
                        {item.label}
                    </Link>
                ))}
            </nav>
        </aside>
    );
}
