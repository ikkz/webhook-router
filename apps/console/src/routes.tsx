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
                to: '/login',
            });
        }
    }
});

export const indexRoute = createRoute({
    getParentRoute: () => protectedRoute,
    path: '/',
    beforeLoad: () => {
        throw redirect({ to: '/endpoints' });
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
