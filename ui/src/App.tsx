import { Router, RouterProvider } from "@tanstack/react-router";

import { LoadingSplashscreen } from "@flow/components";
import { useAuth } from "@flow/lib/auth";

type AppProps = {
  router: Router<any, any>;
};

export function App({ router }: AppProps) {
  const { isLoading } = useAuth();

  if (isLoading) {
    return <LoadingSplashscreen />;
  }

  return <RouterProvider router={router} />;
}
