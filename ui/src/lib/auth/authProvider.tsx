import { Auth0Provider } from "@auth0/auth0-react";
import React, { createContext, ReactNode, useState } from "react";

import {
  config,
  getAuthInfo,
  getSignInCallbackUrl,
  logInToTenant,
} from "@flow/config";

import { useMockAuth } from "./mockAuth";

import { useAuth0Auth, AuthHook } from "./";

export const AuthContext = createContext<AuthHook | null>(null);

const Auth0Wrapper = ({ children }: { children: ReactNode }) => {
  const auth = useAuth0Auth();
  return <AuthContext.Provider value={auth}>{children}</AuthContext.Provider>;
};

const MockAuthWrapper = ({ children }: { children: ReactNode }) => {
  const auth = useMockAuth();
  return <AuthContext.Provider value={auth}>{children}</AuthContext.Provider>;
};

export const AuthProvider: React.FC<{ children?: ReactNode }> = ({
  children,
}) => {
  const [authInfo] = useState(() => {
    logInToTenant(); // note that it includes side effect
    return getAuthInfo();
  });

  // Dev-only, and after the hook above so the hook count never varies.
  // import.meta.env.DEV is statically false in production builds, so this branch
  // and useMockAuth are dropped from the bundle entirely.
  if (import.meta.env.DEV && config().mockEnabled) {
    return <MockAuthWrapper>{children}</MockAuthWrapper>;
  }

  const domain = authInfo?.auth0Domain;
  const clientId = authInfo?.auth0ClientId;
  const audience = authInfo?.auth0Audience;

  return domain && clientId ? (
    <Auth0Provider
      domain={domain}
      clientId={clientId}
      authorizationParams={{
        audience: audience,
        scope: "openid profile email offline_access",
        redirect_uri: getSignInCallbackUrl(),
      }}
      useRefreshTokens
      useRefreshTokensFallback
      cacheLocation="localstorage">
      <Auth0Wrapper>{children}</Auth0Wrapper>
    </Auth0Provider>
  ) : null;
};
