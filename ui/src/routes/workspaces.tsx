import { TooltipProvider } from "@radix-ui/react-tooltip";
import {
  createFileRoute,
  Outlet,
  useNavigate,
  useParams,
} from "@tanstack/react-router";
import { useEffect, useState } from "react";

import { Button, FlowLogo, LoadingSplashscreen } from "@flow/components";
import AuthenticationWrapper from "@flow/features/AuthenticationWrapper";
import { NotificationSystem } from "@flow/features/NotificationSystem";
import { useAuth } from "@flow/lib/auth";
import { GraphQLProvider, useUser } from "@flow/lib/gql";
import { I18nProvider, useT } from "@flow/lib/i18n";
import { ThemeProvider } from "@flow/lib/theme";

export const Route = createFileRoute("/workspaces")({
  component: () => (
    <AuthenticationWrapper>
      <WorkspaceRoute />
    </AuthenticationWrapper>
  ),
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
  console.log("ME", me, isLoading, isError);
  return isLoading ? (
    <LoadingSplashscreen />
  ) : isError || !me || !me?.myWorkspaceId ? (
    <ErrorPage errorMessage={"Could not fetch user"} />
  ) : null;
};

function ErrorPage({ errorMessage }: { errorMessage: string }) {
  const t = useT();
  return (
    <div className="flex h-screen items-center justify-center">
      <div className="flex flex-col items-center gap-10">
        <div className="flex items-center gap-4">
          <div className="rounded bg-logo p-2">
            <FlowLogo className="size-[75px]" />
          </div>
        </div>
        <p className="text-destructive dark:font-extralight">{errorMessage}</p>
        <Button variant="outline" onClick={() => window.location.reload()}>
          <p className="dark:font-extralight">{t("Reload")}</p>
        </Button>
      </div>
    </div>
  );
}
