
import { createRootRoute, createRoute } from '@tanstack/react-router';
import { Layout } from './components/Layout';

export const rootRoute = createRootRoute({
    component: Layout,
});

export const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: function Index() {
        return (
            <div className="p-2">
                <h3>Welcome to the Project Console!</h3>
            </div>
        );
    },
});

export const routeTree = rootRoute.addChildren([indexRoute]);
