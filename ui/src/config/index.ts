declare global {
  let __APP_VERSION__: string;
  interface Window {
    FLOW_CONFIG?: Config;
    FLOW_E2E_ACCESS_TOKEN?: string;
  }
}

export type AuthInfo = {
  auth0ClientId?: string;
  auth0Domain?: string;
  auth0Audience?: string;
  authProvider?: string;
};

export type Config = {
  version?: string;
  brandName?: string;
  devMode?: boolean;
  githubRepoUrl?: string;
  tosUrl?: string;
  documentationUrl?: string;
  multiTenant?: Record<string, AuthInfo>;
} & AuthInfo;

const defaultConfig: Config = {
  version: "X.X.X",
  brandName: "Re:Earth Flow",
  githubRepoUrl: "https://github.com/reearth/reearth-flow",
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

// TODO: Need to check with Kyle on why the `tenant` code
// TODO: Should this be flow_tennat?
const tenantKey = "reearth_tennant";

export function logOutFromTenant() {
  window.localStorage.removeItem(tenantKey);
}

export function e2eAccessToken(): string | undefined {
  return window.FLOW_E2E_ACCESS_TOKEN;
}

// TODO: Move these functions to authInfo.ts (same as reearth)
function getLogginInTenantName(): string | null {
  const path = window.location.pathname;
  // /auth/<tennant-name>
  if (path.startsWith("/auth/")) {
    const name = path.split("/")[2];
    return name || null;
  }
  return null;
}

export function logInToTenant() {
  const tenantName = getLogginInTenantName();
  const q = new URLSearchParams(window.location.search);
  if (tenantName && q.get("code")) {
    window.localStorage.setItem(tenantKey, tenantName);
  }
}

function getTenantName(): string | null {
  const loggingInTenantName = getLogginInTenantName();
  if (loggingInTenantName) {
    return loggingInTenantName;
  }
  return window.localStorage.getItem(tenantKey);
}

export function getSignInCallbackUrl() {
  const tenantName = getTenantName();
  if (tenantName) {
    // multi-tenant
    return `${window.location.origin}/auth/${tenantName}`;
  }
  return window.location.origin;
}

// TODO: Do we really need this code?
function getMultitenantAuthInfo(conf = config()): AuthInfo | undefined {
  if (!conf?.multiTenant) return;
  const name = getTenantName();
  if (name) {
    const tenant = conf.multiTenant[name];
    if (tenant && !tenant.authProvider) {
      tenant.authProvider = "auth0";
    }
    return tenant;
  }
  return;
}

function defaultAuthInfo(conf = config()): AuthInfo | undefined {
  if (!conf) return;
  return {
    auth0Audience: conf.auth0Audience,
    auth0ClientId: conf.auth0ClientId,
    auth0Domain: conf.auth0Domain,
  };
}

export function getAuthInfo(conf = config()): AuthInfo | undefined {
  return getMultitenantAuthInfo(conf) || defaultAuthInfo(conf);
}
