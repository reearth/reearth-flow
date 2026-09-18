import { useAuth0 } from "@auth0/auth0-react";

import { e2eAccessToken, logOutFromTenant } from "@flow/config";

import type { AuthHook } from ".";
import { errorKey } from ".";

export const useAuth0Auth = (): AuthHook => {
  const {
    isAuthenticated,
    error,
    isLoading,
    loginWithRedirect,
    logout,
    getAccessTokenSilently,
    user,
  } = useAuth0();

  return {
    isAuthenticated: !!e2eAccessToken() || (isAuthenticated && !error),
    isLoading,
    error: error?.message ?? null,
    getAccessToken: async () => {
      // auth0-spa-js types getTokenSilently as string | undefined. Returning
      // undefined here would reach callers as a `Bearer undefined` header, so
      // fail loudly instead.
      const token = await getAccessTokenSilently();
      if (!token) throw new Error("Auth0 returned no access token");
      return token;
    },
    login: () => {
      logOutFromTenant();
      return loginWithRedirect();
    },
    logout: () => {
      logOutFromTenant();
      return logout({
        logoutParams: {
          returnTo: error
            ? `${window.location.origin}?${errorKey}=${encodeURIComponent(error?.message)}`
            : window.location.origin,
        },
      });
    },
    user: user,
  };
};
