import { useAuth0 } from "@auth0/auth0-react";
import { useCallback, useMemo } from "react";

import { e2eAccessToken, logOutFromTenant } from "@flow/config";

import { errorKey, AuthHook } from ".";

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

  // Memoize the returned methods and object. Without this, every render produced
  // a new `getAccessToken` reference, which is a dependency of the yjs setup
  // effect (useYjsSetup) and forced the Y.Doc to be recreated on every auth
  // re-render / token refresh — the churn behind the [yjs#509] flood.
  const getAccessToken = useCallback(
    () => getAccessTokenSilently(),
    [getAccessTokenSilently],
  );

  const login = useCallback(() => {
    logOutFromTenant();
    return loginWithRedirect();
  }, [loginWithRedirect]);

  const logoutCb = useCallback(() => {
    logOutFromTenant();
    return logout({
      logoutParams: {
        returnTo: error
          ? `${window.location.origin}?${errorKey}=${encodeURIComponent(error?.message)}`
          : window.location.origin,
      },
    });
  }, [logout, error]);

  return useMemo(
    () => ({
      isAuthenticated: !!e2eAccessToken() || (isAuthenticated && !error),
      isLoading,
      error: error?.message ?? null,
      getAccessToken,
      login,
      logout: logoutCb,
      user,
    }),
    [isAuthenticated, error, isLoading, getAccessToken, login, logoutCb, user],
  );
};
