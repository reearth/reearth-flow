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

export default async function loadConfig() {
  if (window.FLOW_CONFIG) return;

  window.FLOW_CONFIG = {};

  const config: Config = {
    ...(await (await fetch("/flow_config.json")).json()),
  };

  window.FLOW_CONFIG = config;
}

export function config(): Config | undefined {
  return window.FLOW_CONFIG;
}
