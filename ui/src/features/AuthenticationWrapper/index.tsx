import { withAuthenticationRequired } from "@auth0/auth0-react";
import { ReactNode } from "react";

import { useAuthenticationRequired } from "@flow/lib/auth";

const AuthenticationWrapper: React.FC<{ children?: ReactNode }> = ({
  children,
}) => {
  const [isAuthenticated] = useAuthenticationRequired(); // TODO: show error
  return isAuthenticated && children ? <>{children}</> : null;
};

const withAuthorisation = (): ((props: any) => React.FC<any>) => {
  return withAuthenticationRequired as unknown as (props: any) => React.FC<any>;
};

export default withAuthorisation()(AuthenticationWrapper);
