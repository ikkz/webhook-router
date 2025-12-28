import { StrictMode } from 'react';
import * as ReactDOM from 'react-dom/client';
import { RouterProvider } from '@tanstack/react-router';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { router } from './router';
import { client } from '@webhook-router/api-client';
import { AuthProvider, useAuth } from './components/auth-provider';

const queryClient = new QueryClient();

// Configure API client to use stored credentials
// Use current page path to derive API base URL for reverse proxy compatibility
// e.g., if page is at /console, API calls will be /console/api/...
// if page is at /admin/console, API calls will be /admin/console/api/...
// Note: API client already adds /api prefix to all endpoints, so baseUrl should be just the base path
const getApiBaseUrl = () => {
  const path = window.location.pathname;
  // Extract the base path (first segment after /)
  // For /console, /console/endpoints, etc. -> /console
  const basePath = path.split('/').slice(0, 2).join('/') || '';
  return basePath;
};

client.setConfig({
  baseUrl: getApiBaseUrl(),
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
