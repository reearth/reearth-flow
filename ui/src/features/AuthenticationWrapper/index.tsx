import { ReactNode } from "react";

import ErrorPage from "@flow/components/errors/ErrorPage";
import { useAuthenticationRequired } from "@flow/lib/auth";

type Props = {
  children?: ReactNode;
};

const AuthenticationWrapper: React.FC<Props> = ({ children }) => {
  const [isAuthenticated, error] = useAuthenticationRequired();

  if (error) {
    return <ErrorPage errorMessage={error} />;
  }

  return isAuthenticated && children ? children : null;
};

export default AuthenticationWrapper;
