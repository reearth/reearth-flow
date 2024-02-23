import { resolve } from "path";

import react from "@vitejs/plugin-react";
import { UserConfig, defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: [{ find: "@flow", replacement: resolve(__dirname, "./src") }],
  },
  test: {
    environment: "jsdom",
  },
} as UserConfig);
