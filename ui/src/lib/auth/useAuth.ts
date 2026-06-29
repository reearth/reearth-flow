import { useContext, useEffect, useState } from "react";

import { errorKey, AuthContext, useAuth0Auth } from ".";

function useCleanUrl(): [string | undefined, boolean] {
  const { isAuthenticated, isLoading } = useAuth();
  const [error, setError] = useState<string>();
  const [done, setDone] = useState(false);

  useEffect(() => {
    if (isLoading) return;

    const params = new URLSearchParams(window.location.search);

    const error = params.get(errorKey);
    if (error) {
      setError(error);
    }

    params.delete("code");
    params.delete("state");
    params.delete(errorKey);

    const queries = params.toString();
    const url = `${window.location.pathname}${queries ? "?" : ""}${queries}`;

    history.replaceState(null, document.title, url);

    setDone(true);
  }, [isAuthenticated, isLoading]);

  return [error, done];
}

export const useAuth = () => {
  let auth = useContext(AuthContext);

  if (!auth) {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    auth = useAuth0Auth();
  }

  return auth;
};

export function useAuthenticationRequired(): {
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | undefined;
} {
  const {
    isAuthenticated,
    isLoading,
    error: authError,
    login,
    logout,
  } = useAuth();

  const [error, done] = useCleanUrl();

  useEffect(() => {
    if (isLoading || isAuthenticated || !done) {
      return;
    }

    if (authError) {
      logout();
      return;
    }

    if (error) {
      return;
    }

    login();
  }, [authError, isAuthenticated, isLoading, login, logout, error, done]);

  return { isAuthenticated, isLoading, error };
}
