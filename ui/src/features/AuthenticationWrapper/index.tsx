import { ReactNode } from "react";

import { LoadingSplashscreen } from "@flow/components";
import ErrorPage from "@flow/components/errors/ErrorPage";
import { useAuthenticationRequired } from "@flow/lib/auth";

type Props = {
  children?: ReactNode;
};

const AuthenticationWrapper: React.FC<Props> = ({ children }) => {
  const { isAuthenticated, isLoading, error } = useAuthenticationRequired();

  if (isAuthenticated) {
    return children ? children : null;
  }

  if (isLoading) {
    return <LoadingSplashscreen />;
  }

  if (error) {
    return <ErrorPage errorMessage={error} />;
  }

  return null;
};

export default AuthenticationWrapper;
