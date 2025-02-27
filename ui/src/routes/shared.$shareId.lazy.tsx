import { createLazyFileRoute } from "@tanstack/react-router";
import { ReactFlowProvider, useReactFlow } from "@xyflow/react";

import { LoadingSplashscreen } from "@flow/components";
import SharedCanvas from "@flow/features/SharedCanvas";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useFullscreen, useShortcuts } from "@flow/hooks";
import useYjsSetup from "@flow/lib/yjs/useYjsSetup";

export const Route = createLazyFileRoute("/shared/$shareId")({
  component: () => (
    <ReactFlowProvider>
      <EditorComponent />
    </ReactFlowProvider>
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

  const { state, isSynced } = useYjsSetup({
    workflowId: DEFAULT_ENTRY_GRAPH_ID,
  });

  return !state || !isSynced ? (
    <LoadingSplashscreen />
  ) : (
    <SharedCanvas yWorkflows={state.yWorkflows} />
  );
};
