import { createLazyFileRoute, useParams } from "@tanstack/react-router";
import { ReactFlowProvider, useReactFlow } from "@xyflow/react";
import { useEffect, useState } from "react";
import { useHotkeys } from "react-hotkeys-hook";

import {
  FlowLogo,
  LoadingSplashscreen,
  TooltipProvider,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import ErrorPage from "@flow/components/errors/ErrorPage";
import { ProjectCorruptionError } from "@flow/errors";
import AuthenticationWrapper from "@flow/features/AuthenticationWrapper";
import NotFound from "@flow/features/NotFound";
import SharedCanvas from "@flow/features/SharedCanvas";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useFullscreen } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import { GraphQLProvider, useSharedProject } from "@flow/lib/gql";
import { I18nProvider, useT } from "@flow/lib/i18n";
import { ThemeProvider } from "@flow/lib/theme";
import useYjsSetup from "@flow/lib/yjs/useYjsSetup";

export const Route = createLazyFileRoute("/shared/$sharedToken")({
  component: () => <SharedRoute />,
  errorComponent: ({ error }) => <ErrorComponent error={error} />,
  notFoundComponent: () => <NotFound />,
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
  const t = useT();
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  const { handleFullscreenToggle } = useFullscreen();

  const globalHotKeys = ["+", "-", "meta+0", "ctrl+0", "f", "meta+f", "ctrl+f"];

  useHotkeys(
    globalHotKeys,
    (_, handler) => {
      switch (handler.keys?.join("")) {
        case "+":
          zoomIn();
          break;
        case "-":
          zoomOut();
          break;
        case "0":
          fitView();
          break;
        case "f":
          handleFullscreenToggle();
          break;
      }
    },
    { preventDefault: true },
  );

  const { useGetSharedProject } = useSharedProject();

  const { sharedToken } = useParams({ strict: false });

  const { sharedProject, isError } = useGetSharedProject(sharedToken);

  const { yWorkflows, yDocState, isSynced, undoTrackerActionWrapper } =
    useYjsSetup({
      projectId: sharedProject?.id,
      workflowId: DEFAULT_ENTRY_GRAPH_ID,
    });

  return isError ? (
    <ErrorPage errorMessage={t("Please check the shared URL is correct.")} />
  ) : !yWorkflows ||
    !isSynced ||
    !undoTrackerActionWrapper ||
    !sharedProject ? (
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

const ErrorComponent = ({ error }: { error: Error }) => {
  const t = useT();

  return (
    <>
      {error instanceof ProjectCorruptionError ? (
        <div className="flex h-screen w-full flex-col items-center justify-center">
          <BasicBoiler
            text={t("Project or version is corrupted.")}
            icon={<FlowLogo className="size-16 text-accent" />}
          />
        </div>
      ) : (
        <ErrorPage />
      )}
    </>
  );
};
