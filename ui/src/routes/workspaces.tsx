import { TooltipProvider } from "@radix-ui/react-tooltip";
import {
  createFileRoute,
  Outlet,
  useNavigate,
  useParams,
} from "@tanstack/react-router";
import { useEffect, useState } from "react";

import { LoadingSplashscreen } from "@flow/components";
import ErrorPage from "@flow/components/errors/ErrorPage";
import AuthenticationWrapper from "@flow/features/AuthenticationWrapper";
import NotFound from "@flow/features/NotFound";
import { NotificationSystem } from "@flow/features/NotificationSystem";
import { useAuth } from "@flow/lib/auth";
import { GraphQLProvider, useUser } from "@flow/lib/gql";
import { I18nProvider } from "@flow/lib/i18n";
import { ThemeProvider } from "@flow/lib/theme";

export const Route = createFileRoute("/workspaces")({
  component: () => (
    <AuthenticationWrapper>
      <WorkspaceRoute />
    </AuthenticationWrapper>
  ),
  errorComponent: () => <ErrorPage />,
  notFoundComponent: () => <NotFound />,
});

const WorkspaceRoute = () => {
  const { getAccessToken } = useAuth();

  const [accessToken, setAccessToken] = useState<string | undefined>(undefined);

  useEffect(() => {
    (async () => {
      const token = await getAccessToken();
      setAccessToken(token);
    })();
  }, [getAccessToken]);

  return accessToken ? (
    <ThemeProvider>
      <GraphQLProvider gqlAccessToken={accessToken}>
        <I18nProvider>
          <TooltipProvider>
            <NotificationSystem />
            <Outlet />
            <WorkspaceNavigation />
          </TooltipProvider>
        </I18nProvider>
      </GraphQLProvider>
    </ThemeProvider>
  ) : (
    <LoadingSplashscreen />
  );
};

const WorkspaceNavigation = () => {
  const { workspaceId } = useParams({ strict: false });
  const navigate = useNavigate();
  const { useGetMe } = useUser();
  const { me, isLoading, isError } = useGetMe();

  useEffect(() => {
    if (!me || !me?.myWorkspaceId || workspaceId) return;
    navigate({
      to: `/workspaces/${me?.myWorkspaceId}/projects`,
      replace: true,
    });
  }, [me, workspaceId, navigate]);

  return isLoading ? (
    <LoadingSplashscreen />
  ) : isError || !me || !me?.myWorkspaceId ? (
    <ErrorPage errorMessage={"Could not fetch user"} />
  ) : null;
};
