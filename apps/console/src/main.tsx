import { StrictMode } from 'react';
import * as ReactDOM from 'react-dom/client';
import { RouterProvider } from '@tanstack/react-router';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { router } from './router';
import { client } from '@webhook-router/api-client';
import { AuthProvider, useAuth } from './components/auth-provider';

const queryClient = new QueryClient();

// Configure API client to use stored credentials
client.setConfig({
  baseUrl: '',
  auth: async () => {
    const token = localStorage.getItem('webhook_router_auth');
    return token ? `Basic ${token}` : '';
  }
});

function InnerApp() {
  const auth = useAuth();
  return <RouterProvider router={router} context={{ auth }} />;
}

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement,
);

root.render(
  <StrictMode>
    <AuthProvider>
      <QueryClientProvider client={queryClient}>
        <InnerApp />
      </QueryClientProvider>
    </AuthProvider>
  </StrictMode>,
);
