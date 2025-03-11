import { createLazyFileRoute, useParams } from "@tanstack/react-router";
import { ReactFlowProvider, useReactFlow } from "@xyflow/react";

import { LoadingSplashscreen } from "@flow/components";
import SharedCanvas from "@flow/features/SharedCanvas";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useFullscreen, useShortcuts } from "@flow/hooks";
import { GraphQLProvider, useSharedProject } from "@flow/lib/gql";
import { I18nProvider } from "@flow/lib/i18n";
import { ThemeProvider } from "@flow/lib/theme";
import useYjsSetup from "@flow/lib/yjs/useYjsSetup";

export const Route = createLazyFileRoute("/shared/$sharedToken")({
  component: () => (
    <ThemeProvider>
      <GraphQLProvider>
        <I18nProvider>
          <ReactFlowProvider>
            <EditorComponent />
          </ReactFlowProvider>
        </I18nProvider>
      </GraphQLProvider>
    </ThemeProvider>
  ),
});

const EditorComponent = () => {
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

  const { yDoc, isSynced, undoTrackerActionWrapper } = useYjsSetup({
    projectId: sharedProject?.id,
    workflowId: DEFAULT_ENTRY_GRAPH_ID,
  });

  return !yDoc || !isSynced || !undoTrackerActionWrapper ? (
    <LoadingSplashscreen />
  ) : (
    <div className="h-screen">
      <SharedCanvas
        yDoc={yDoc}
        project={sharedProject}
        undoTrackerActionWrapper={undoTrackerActionWrapper}
      />
    </div>
  );
};
