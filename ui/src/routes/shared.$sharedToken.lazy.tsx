import { createLazyFileRoute, useParams } from "@tanstack/react-router";
import { ReactFlowProvider, useReactFlow } from "@xyflow/react";
import { useEffect, useState } from "react";

import {
  FlowLogo,
  LoadingSplashscreen,
  TooltipProvider,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import AuthenticationWrapper from "@flow/features/AuthenticationWrapper";
import SharedCanvas from "@flow/features/SharedCanvas";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useFullscreen, useShortcuts } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import { GraphQLProvider, useSharedProject } from "@flow/lib/gql";
import { I18nProvider, useT } from "@flow/lib/i18n";
import { ThemeProvider } from "@flow/lib/theme";
import useYjsSetup from "@flow/lib/yjs/useYjsSetup";

export const Route = createLazyFileRoute("/shared/$sharedToken")({
  component: () => <SharedRoute />,
  errorComponent: () => <ErrorComponent />,
});

const SharedRoute = () => {
  const [accessToken, setAccessToken] = useState<string | undefined>(undefined);

  const { getAccessToken } = useAuth();

  useEffect(() => {
    if (!accessToken) {
      (async () => {
        const token = await getAccessToken();
        setAccessToken(token);
      })();
    }
  }, [accessToken, getAccessToken]);

  return accessToken ? (
    <AuthenticationWrapper>
      <ThemeProvider>
        <GraphQLProvider gqlAccessToken={accessToken}>
          <I18nProvider>
            <TooltipProvider>
              <ReactFlowProvider>
                <EditorComponent accessToken={accessToken} />
              </ReactFlowProvider>
            </TooltipProvider>
          </I18nProvider>
        </GraphQLProvider>
      </ThemeProvider>
    </AuthenticationWrapper>
  ) : (
    <ThemeProvider>
      <GraphQLProvider>
        <I18nProvider>
          <TooltipProvider>
            <ReactFlowProvider>
              <EditorComponent />
            </ReactFlowProvider>
          </TooltipProvider>
        </I18nProvider>
      </GraphQLProvider>
    </ThemeProvider>
  );
};

const EditorComponent = ({ accessToken }: { accessToken?: string }) => {
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  const { handleFullscreenToggle } = useFullscreen();
  useShortcuts([
    {
      keyBinding: { key: "+", commandKey: false },
      callback: zoomIn,
    },
    {
      keyBinding: { key: "-", commandKey: false },
      callback: zoomOut,
    },
    {
      keyBinding: { key: "0", commandKey: true },
      callback: fitView,
    },
    {
      keyBinding: { key: "f", commandKey: true },
      callback: handleFullscreenToggle,
    },
  ]);

  const { useGetSharedProject } = useSharedProject();

  const { sharedToken } = useParams({ strict: false });

  const { sharedProject } = useGetSharedProject(sharedToken);

  const { yWorkflows, yDocState, isSynced, undoTrackerActionWrapper } =
    useYjsSetup({
      projectId: sharedProject?.id,
      workflowId: DEFAULT_ENTRY_GRAPH_ID,
    });

  return !yWorkflows || !isSynced || !undoTrackerActionWrapper ? (
    <LoadingSplashscreen />
  ) : (
    <SharedCanvas
      yWorkflows={yWorkflows}
      yDoc={yDocState}
      project={sharedProject}
      accessToken={accessToken}
      undoTrackerActionWrapper={undoTrackerActionWrapper}
    />
  );
};

const ErrorComponent = () => {
  const t = useT();

  return (
    <div className="flex flex-col h-screen w-full items-center justify-center">
      <BasicBoiler
        text={t("Project or version is corrupted.")}
        icon={<FlowLogo className="size-16 text-accent" />}
      />
    </div>
  );
};
