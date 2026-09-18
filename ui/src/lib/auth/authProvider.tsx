import { Auth0Provider } from "@auth0/auth0-react";
import type { ReactNode } from "react";
import React, { createContext, useState } from "react";

import { getAuthInfo, getSignInCallbackUrl, logInToTenant } from "@flow/config";

import type { AuthHook } from "./";
import { useAuth0Auth } from "./";

export const AuthContext = createContext<AuthHook | null>(null);

const Auth0Wrapper = ({ children }: { children: ReactNode }) => {
  const auth = useAuth0Auth();
  return <AuthContext.Provider value={auth}>{children}</AuthContext.Provider>;
};

export const AuthProvider: React.FC<{ children?: ReactNode }> = ({
  children,
}) => {
  const [authInfo] = useState(() => {
    logInToTenant(); // note that it includes side effect
    return getAuthInfo();
  });

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
