import { createRootRoute, createRoute, redirect, Outlet } from '@tanstack/react-router';
import { Layout } from './components/layout';
import { EndpointsPage } from './routes/endpoints';
import { EndpointDetailsPage } from './routes/endpoint-details';
import { EventsPage } from './routes/events';
import { LoginPage } from './routes/login';

export const rootRoute = createRootRoute({
    component: () => <Outlet />,
});

export const loginRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/login',
    component: LoginPage,
});

const protectedRoute = createRoute({
    getParentRoute: () => rootRoute,
    id: 'protected',
    component: Layout,
    beforeLoad: ({ context }) => {
        const { auth } = context as any;
        if (!auth.isAuthenticated) {
            throw redirect({
                // @ts-expect-error Types not yet generated for /login
                to: '/login',
            });
        }
    }
});

export const indexRoute = createRoute({
    getParentRoute: () => protectedRoute,
    path: '/',
    component: function Index() {
        return (
            <div className="p-4">
                <h3 className="text-2xl font-bold mb-4">Welcome to Webhook Router Console</h3>
                <p className="text-muted-foreground">Manage your endpoints, targets, and view events using the sidebar.</p>
            </div>
        );
    },
});

export const endpointsRoute = createRoute({
    getParentRoute: () => protectedRoute,
    path: '/endpoints',
    component: EndpointsPage,
});

export const endpointDetailsRoute = createRoute({
    getParentRoute: () => protectedRoute,
    path: '/endpoints/$endpointId',
    component: EndpointDetailsPage,
});

export const eventsRoute = createRoute({
    getParentRoute: () => protectedRoute,
    path: '/events',
    component: EventsPage,
});

export const routeTree = rootRoute.addChildren([
    loginRoute,
    protectedRoute.addChildren([
        indexRoute,
        endpointsRoute,
        endpointDetailsRoute,
        eventsRoute,
    ]),
]);
