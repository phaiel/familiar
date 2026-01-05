import { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'com.familiar.app',
  appName: 'Familiar',
  webDir: 'out',
  server: {
    // For development, point to your Next.js dev server
    // For production, this will use the built static files
    // url: 'http://localhost:3000',
    // cleartext: true, // Allow HTTP (only for development)
  },
  ios: {
    contentInset: 'automatic',
    scrollEnabled: true,
  },
  plugins: {
    Keyboard: {
      resize: 'body',
      style: 'dark',
      resizeOnFullScreen: true,
    },
    StatusBar: {
      style: 'dark',
    },
  },
};

export default config;
