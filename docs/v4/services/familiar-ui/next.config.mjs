/** @type {import('next').NextConfig} */
const nextConfig = {
  // For Capacitor static export (optional - uncomment if using static export)
  // output: 'export',
  
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://127.0.0.1:3001/api/:path*', // Proxy to familiar-api (IPv4 explicit)
      },
    ]
  },
};

export default nextConfig;
