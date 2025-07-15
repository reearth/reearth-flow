import { withAuthenticationRequired } from "@auth0/auth0-react";
import { ReactNode } from "react";

import { useAuthenticationRequired } from "@flow/lib/auth";

type Props = {
  children?: ReactNode;
};

const AuthenticationWrapper: React.FC<Props> = ({ children }) => {
  const [isAuthenticated] = useAuthenticationRequired(); // TODO: show error
  console.log("AuthenticationWrapper: isAuthenticated", isAuthenticated);
  return isAuthenticated && children ? children : null;
};

const withAuthorization = (): ((
  component: React.FC<Props>,
) => React.FC<Props>) => {
  console.log("asldkfjaslkdfjddd");
  return withAuthenticationRequired as unknown as (
    component: React.FC<Props>,
  ) => React.FC<Props>;
};

export default withAuthorization()(AuthenticationWrapper);
