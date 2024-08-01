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
    auth = useAuth0Auth();
  }

  return auth;
};

export function useAuthenticationRequired(): [boolean, string | undefined] {
  const { isAuthenticated, isLoading, error: authError, login, logout } = useAuth();

  useEffect(() => {
    if (isLoading || isAuthenticated) {
      return;
    }

    if (authError) {
      logout();
      return;
    }

    login();
  }, [authError, isAuthenticated, isLoading, login, logout]);

  const [error] = useCleanUrl();

  return [isAuthenticated, error];
}
