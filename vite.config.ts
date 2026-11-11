import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

// The Tauri dev server must not fall back to a random port: tauri.conf.json
// pins devUrl to 1420.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host ?? false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    target: "chrome110",
    sourcemap: true,
    outDir: "dist",
    emptyOutDir: true,
  },
});
