/// <reference types="vitest" />
/// <reference types="vite/client" />

import { resolve } from "path";

import react from "@vitejs/plugin-react";
import { UserConfig, defineConfig } from "vite";

export default defineConfig(() => {
  return {
    server: {
      port: 3000,
    },
    envPrefix: ["FLOW_", "TAURI_"],
    plugins: [
      react(),
    ],
    build: {
      target: "esnext",
      assetsDir: "static", // avoid conflicts with backend asset endpoints
      rollupOptions: {
        input: {
          main: resolve(__dirname, "index.html"),
        },
      },
      minify: "esbuild",
    },
    resolve: {
      alias: [{ find: "@flow", replacement: resolve(__dirname, "./src") }],
    },
    test: {
      environment: "jsdom",
      setupFiles: ["./src/testing/setup.ts"],
      globals: true,
      coverage: {
        reporter: ["text"],
        include: ["src/**/*.{ts, tsx}"],
        exclude: ["/node_modules/", "/testing/", "src/**/*.test.ts"],
      },
    },
  } as UserConfig;
});
