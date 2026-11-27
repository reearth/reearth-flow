import { createLazyFileRoute } from "@tanstack/react-router";
import { ReactFlowProvider } from "@xyflow/react";
import { useCallback, useEffect, useState } from "react";

import { LoadingSplashscreen, TooltipProvider } from "@flow/components";
import EmbedCanvas from "@flow/features/EmbedCanvas";
import { useAuth } from "@flow/lib/auth";
import type { EmbedInboundMessage, EmbedTheme } from "@flow/lib/embedBridge";
import { useEmbedBridge } from "@flow/lib/embedBridge";
import { GraphQLProvider } from "@flow/lib/gql";
import { I18nProvider } from "@flow/lib/i18n";
import { ThemeProvider, useTheme } from "@flow/lib/theme";

export const Route = createLazyFileRoute("/embed/$projectId")({
  component: () => (
    <ThemeProvider>
      <EmbedRoute />
    </ThemeProvider>
  ),
});

type Session = { accessToken?: string; theme?: EmbedTheme };

const useEmbedSession = (): Session | undefined => {
  const [session, setSession] = useState<Session | undefined>();
  const { getAccessToken } = useAuth();

  const handleMessage = useCallback((message: EmbedInboundMessage) => {
    if (message.type === "flow:init")
      setSession({ accessToken: message.token, theme: message.theme });
  }, []);

  const { isConnected, post } = useEmbedBridge(handleMessage);

  useEffect(() => {
    if (isConnected) {
      post({ type: "flow:ready" });
      return;
    }
    let cancelled = false;
    (async () => {
      const accessToken = await getAccessToken().catch(() => undefined);
      if (!cancelled) setSession({ accessToken });
    })();
    return () => {
      cancelled = true;
    };
  }, [isConnected, post, getAccessToken]);

  return session;
};

const EmbedRoute = () => {
  const { projectId } = Route.useParams();
  const session = useEmbedSession();
  const { previewTheme } = useTheme();

  useEffect(() => {
    if (session?.theme) previewTheme(session.theme);
  }, [session?.theme, previewTheme]);

  if (!session) return <LoadingSplashscreen />;

  return (
    <GraphQLProvider gqlAccessToken={session.accessToken}>
      <I18nProvider>
        <TooltipProvider>
          <ReactFlowProvider>
            <EmbedCanvas projectId={projectId} />
          </ReactFlowProvider>
        </TooltipProvider>
      </I18nProvider>
    </GraphQLProvider>
  );
};
