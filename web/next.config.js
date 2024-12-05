module.exports = {
  async rewrites() {
    return [
      {
        source: "/v1/whip",
        destination: "http://localhost:9090/v1/whip",
      },
      {
        source: "/v1/hls",
        destination: "http://localhost:9090/v1/hls",
      },
      {
        source: "/v1/llhls/:path*",
        destination: "http://localhost:9090/v1/llhls/:path*",
      },
      {
        source: "/v1/public/:path*",
        destination: "http://localhost:9090/v1/public/:path*",
      },
    ];
  },
};
