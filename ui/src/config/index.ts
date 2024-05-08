export type Config = {
  version?: string;
  brandName?: string;
  githubRepoUrl?: string;
  tosUrl?: string;
  documentationUrl?: string;
};

declare global {
  let __APP_VERSION__: string;
  interface Window {
    FLOW_CONFIG?: Config;
  }
}

const defaultConfig: Config = {
  version: "X.X.X",
};

export default async function loadConfig() {
  if (window.FLOW_CONFIG) return;

  window.FLOW_CONFIG = defaultConfig;

  const config: Config = {
    ...(await (await fetch("/flow_config.json")).json()),
  };

  window.FLOW_CONFIG = config;
}

export function config(): Config {
  return window.FLOW_CONFIG ?? {};
}
