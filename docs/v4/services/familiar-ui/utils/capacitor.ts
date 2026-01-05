/**
 * Capacitor utilities - detects if running in Capacitor and provides API URL helpers
 * This allows the app to work both in browser and Capacitor with minimal changes
 */

import { Capacitor } from '@capacitor/core';

export const isCapacitor = Capacitor.isNativePlatform();

/**
 * Get the API base URL
 * In Capacitor, we need to use the actual server URL instead of relative paths
 */
export function getApiUrl(): string {
  if (isCapacitor) {
    // For Capacitor, point to your actual server
    // Change this to your production API URL or use environment variable
    const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';
    return apiUrl;
  }
  
  // In browser, use relative paths (Next.js rewrites handle it)
  return '';
}

/**
 * Make an API call that works in both browser and Capacitor
 */
export async function apiFetch(path: string, options?: RequestInit): Promise<Response> {
  const baseUrl = getApiUrl();
  const url = baseUrl ? `${baseUrl}${path}` : path;
  
  return fetch(url, options);
}
