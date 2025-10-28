import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  async redirects() {
    return [
      {
        source: "/app",
        destination: "/",
        permanent: true,
      },
      {
        source: "/app/:path*",
        destination: "/:path*",
        permanent: true,
      },
      {
        source: "/signup",
        destination: "/register",
        permanent: true,
      },
    ];
  },
};

export default nextConfig;
