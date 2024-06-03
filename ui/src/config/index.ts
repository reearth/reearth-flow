import { AuthInfo } from "./authInfo";

declare global {
  let __APP_VERSION__: string;
  interface Window {
    FLOW_CONFIG?: Config;
    FLOW_E2E_ACCESS_TOKEN?: string;
  }
}

export type Config = {
  version?: string;
  brandName?: string;
  devMode?: boolean;
  tosUrl?: string;
  documentationUrl?: string;
  multiTenant?: Record<string, AuthInfo>;
  api?: string;
} & AuthInfo;

const defaultConfig: Config = {
  version: "X.X.X",
  brandName: "Re:Earth Flow",
};

export default async function loadConfig() {
  if (window.FLOW_CONFIG) return;

  window.FLOW_CONFIG = defaultConfig;

  const config: Config = {
    ...defaultConfig,
    ...(await (await fetch("/flow_config.json")).json()),
  };

  if (window.FLOW_CONFIG.brandName) {
    document.title = window.FLOW_CONFIG.brandName + " v" + config.version;
  }

  window.FLOW_CONFIG = config;
}

export function config(): Config {
  return window.FLOW_CONFIG ?? {};
}

export * from "./authInfo";
