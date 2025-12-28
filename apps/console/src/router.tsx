
import { createRouter, createHashHistory } from '@tanstack/react-router';
import { routeTree } from './routes';

// Create a new router instance
export const router = createRouter({
    routeTree,
    history: createHashHistory(),
    context: {
        auth: undefined!, // This will be set in RouterProvider
    },
});

// Register the router instance for type safety
declare module '@tanstack/react-router' {
    interface Register {
        router: typeof router;
    }
}
