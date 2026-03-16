/**
 * Main entry point for the OCSF Semantic Model Editor.
 * 
 * Configures:
 * - TanStack Query for API data fetching with retry logic
 * - React DnD for drag-and-drop functionality
 * - Global error handling
 * 
 * Requirements: 7.1, 7.2, 7.3, 7.4
 */

import React from 'react';
import ReactDOM from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DndProvider } from 'react-dnd';
import { HTML5Backend } from 'react-dnd-html5-backend';
import App from './App';
import './index.css';

/**
 * Configure TanStack Query client with:
 * - Default stale time of 5 minutes for caching
 * - Retry logic with exponential backoff
 * - Error handling configuration
 */
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Cache data for 5 minutes before considering it stale
      staleTime: 5 * 60 * 1000,
      // Keep unused data in cache for 30 minutes
      gcTime: 30 * 60 * 1000,
      // Retry failed requests up to 3 times
      retry: 3,
      // Exponential backoff for retries (1s, 2s, 4s)
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
      // Don't refetch on window focus by default
      refetchOnWindowFocus: false,
    },
    mutations: {
      // Retry mutations up to 2 times
      retry: 2,
      retryDelay: 1000,
    },
  },
});

// Global error handler for unhandled promise rejections
window.addEventListener('unhandledrejection', (event) => {
  console.error('Unhandled promise rejection:', event.reason);
});

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <DndProvider backend={HTML5Backend}>
        <App />
      </DndProvider>
    </QueryClientProvider>
  </React.StrictMode>
);
