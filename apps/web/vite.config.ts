import path from "node:path";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(__dirname, "../..");

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@arcaea-viewer/wasm": path.resolve(
        workspaceRoot,
        "crates/wasm/pkg/arcaea_viewer_wasm.js",
      ),
    },
  },
  server: {
    fs: {
      allow: [workspaceRoot],
    },
  },
  preview: {
    port: 4173,
  },
});
