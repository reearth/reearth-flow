import { AuthInfo } from "./authInfo";

declare global {
  let __APP_VERSION__: string;
  // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
  interface Window {
    REEARTH_CONFIG?: Config;
    FLOW_E2E_ACCESS_TOKEN?: string;
  }
}

export type Config = {
  version?: string;
  brandLogoUrl?: string;
  brandName?: string;
  devMode?: boolean;
  mockEnabled?: boolean;
  enableWebRTC?: boolean;
  tosUrl?: string;
  documentationUrl?: string;
  multiTenant?: Record<string, AuthInfo>;
  api?: string;
  websocket?: string;
  websocketToken?: string;
} & AuthInfo;

const defaultConfig: Config = {
  version: "X.X.X",
  brandName: "Re:Earth Flow",
};

export default async function loadConfig() {
  if (window.REEARTH_CONFIG) return;

  window.REEARTH_CONFIG = defaultConfig;

  const rawConfig = await (await fetch("/reearth_config.json")).json();

  const config: Config = {
    ...defaultConfig,
    ...rawConfig,
    // Convert string "true"/"false" to boolean for GCP deployment compatibility
    devMode: rawConfig.devMode === "true" || rawConfig.devMode === true,
    mockEnabled:
      rawConfig.mockEnabled === "true" || rawConfig.mockEnabled === true,
    enableWebRTC:
      rawConfig.enableWebRTC === "true" || rawConfig.enableWebRTC === true,
  };

  window.REEARTH_CONFIG = config;
}

export function config(): Config {
  return window.REEARTH_CONFIG ?? {};
}

export * from "./authInfo";
