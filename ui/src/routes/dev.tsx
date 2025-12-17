import { TooltipProvider } from "@radix-ui/react-tooltip";
import { createFileRoute, notFound } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import { LoadingSplashscreen } from "@flow/components";
import { config } from "@flow/config";
import AuthenticationWrapper from "@flow/features/AuthenticationWrapper";
import Dev from "@flow/features/Dev";
import { NotificationSystem } from "@flow/features/NotificationSystem";
import { useAuth } from "@flow/lib/auth";
import { GraphQLProvider } from "@flow/lib/gql";
import { I18nProvider } from "@flow/lib/i18n";
import { ThemeProvider } from "@flow/lib/theme";

export const Route = createFileRoute("/dev")({
  component: () => (
    <AuthenticationWrapper>
      <DevRoute />
    </AuthenticationWrapper>
  ),
  loader: async () => {
    const { devMode } = config();
    if (!devMode) {
      throw notFound();
    }
    return { devMode };
  },
});

function DevRoute() {
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
            <Dev />
          </TooltipProvider>
        </I18nProvider>
      </GraphQLProvider>
    </ThemeProvider>
  ) : (
    <LoadingSplashscreen />
  );
}
