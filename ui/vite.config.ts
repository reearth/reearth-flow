/// <reference types="vitest" />
/// <reference types="vite/client" />

import { readFileSync } from "fs";
import { resolve } from "path";

import tailwindcss from "@tailwindcss/vite";
import { TanStackRouterVite } from "@tanstack/router-vite-plugin";
import react from "@vitejs/plugin-react";
import { readEnv } from "read-env";
import { Plugin, UserConfig, defineConfig, loadEnv } from "vite";
import cesium from "vite-plugin-cesium";

import pkg from "./package.json";

export default defineConfig(() => {
  return {
    server: {
      port: 3000,
    },
    envPrefix: "FLOW_",
    plugins: [react(), TanStackRouterVite(), cesium(), config(), tailwindcss()],
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

function config(): Plugin {
  return {
    name: "reearth_config",
    async configureServer(server) {
      const envs = loadEnv(
        server.config.mode,
        server.config.envDir ?? process.cwd(),
        server.config.envPrefix,
      );
      const remoteConfig = envs.REEARTH_CONFIG_URL
        ? await (await fetch(envs.REEARTH_CONFIG_URL)).json()
        : {};

      const configRes = JSON.stringify(
        {
          version: pkg.version,
          ...remoteConfig,
          ...readEnv("FLOW", {
            source: envs,
          }),
          ...loadJSON("./reearth_config.json"),
        },
        null,
        2,
      );

      server.middlewares.use((req, res, next) => {
        if (req.method === "GET" && req.url === "/reearth_config.json") {
          res.statusCode = 200;
          res.setHeader("Content-Type", "application/json");
          res.write(configRes);
          res.end();
        } else {
          next();
        }
      });
    },
  };
}

function loadJSON(path: string) {
  try {
    return JSON.parse(readFileSync(path, "utf8")) || {};
  } catch (_err) {
    return {};
  }
}
