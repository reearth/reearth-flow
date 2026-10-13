/// <reference types="vitest" />
/// <reference types="vite/client" />

import { readdirSync, readFileSync } from "fs";
import { createRequire } from "module";
import { dirname, resolve } from "path";

import tailwindcss from "@tailwindcss/vite";
import { TanStackRouterVite } from "@tanstack/router-vite-plugin";
import react from "@vitejs/plugin-react";
import { readEnv } from "read-env";
import type { Plugin, UserConfig } from "vite";
import { defineConfig, loadEnv } from "vite";
import svgr from "vite-plugin-svgr";

import pkg from "./package.json" with { type: "json" };

export default defineConfig(() => {
  return {
    server: {
      port: 3000,
    },
    envPrefix: "FLOW_",
    plugins: [
      svgr(),
      react(),
      TanStackRouterVite(),
      config(),
      tailwindcss(),
      navaraWorkerWasm(),
    ],
    build: {
      target: "esnext",
      assetsDir: "static", // avoid conflicts with backend asset endpoints
      rollupOptions: {
        input: {
          main: resolve(import.meta.dirname, "index.html"),
        },
        external: ["./src/mocks/**"],
      },
      minify: "esbuild",
    },
    optimizeDeps: {
      // The map engine's WebAssembly modules locate their .wasm beside their
      // own code. Pre-bundled for the dev server, that code moves and the file
      // is no longer beside it, so these are served as published instead.
      exclude: [
        "@navaramap/engine",
        "@navaramap/engine-worker",
        "@navaramap/engine-font-worker",
      ],
    },
    resolve: {
      alias: [
        { find: "@flow", replacement: resolve(import.meta.dirname, "./src") },
      ],
    },
    test: {
      environment: "jsdom",
      setupFiles: ["./src/testing/setup.ts"],
      globals: true,
      include: ["src/**/*.{test,spec}.{ts,tsx}"],
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

// The map engine's prebuilt workers load their .wasm (the Draco decoder among
// them) from beside themselves, by names fixed inside the workers. The build
// moves the workers into `static/` but cannot see those names, so it leaves
// the .wasm behind; without them, compressed 3D Tiles never draw. They are
// copied beside the workers here, unrenamed. The dev server needs none of
// this: it serves the workers from beside their .wasm.
function navaraWorkerWasm(): Plugin {
  const assetsDir = resolve(
    dirname(createRequire(import.meta.url).resolve("@navaramap/three")),
    "assets",
  );
  return {
    name: "navara_worker_wasm",
    apply: "build",
    generateBundle(_, bundle) {
      for (const name of readdirSync(assetsDir)) {
        const fileName = `static/${name}`;
        // One may already be emitted, by a package that shares its content.
        if (!name.endsWith(".wasm") || fileName in bundle) continue;
        this.emitFile({
          type: "asset",
          fileName,
          source: readFileSync(resolve(assetsDir, name)),
        });
      }
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
