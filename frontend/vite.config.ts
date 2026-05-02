import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 5173,
    // Remove body size limit so large PDFs can be proxied
    proxy: {
      "/api": {
        target: "http://127.0.0.1:3847",
        changeOrigin: true,
        configure: (proxy) => {
          proxy.on("error", (err) => {
            console.error("[proxy error]", err.message);
          });
          proxy.on("proxyReq", (_proxyReq, req) => {
            // Log for debugging
            if (req.url?.startsWith("/api/pdf")) {
              console.log(`[proxy] ${req.method} ${req.url}`);
            }
          });
        },
      },
      "/ws": {
        target: "ws://127.0.0.1:3847",
        ws: true,
      },
    },
  },
  build: {
    outDir: "dist",
    sourcemap: false,
  },
});
