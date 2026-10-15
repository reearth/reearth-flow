import { useReactFlow, useStore, useStoreApi } from "@xyflow/react";
import type { ReactFlowState } from "@xyflow/react";
import { useEffect, useRef } from "react";

const FIT_VIEW_PADDING = 0.5;

/** Fit anyway if some node never reports a size, so the switch still lands. */
const MEASURE_TIMEOUT = 500;

const allNodesMeasured = ({
  nodeLookup,
}: Pick<ReactFlowState, "nodeLookup">) => {
  for (const node of nodeLookup.values()) {
    if (!node.hidden && !(node.measured.width && node.measured.height)) {
      return false;
    }
  }
  return true;
};

/**
 * Fits the canvas to the workflow being opened, once its nodes have a size.
 *
 * A node that has never been drawn is stored as 0×0. ReactFlow counts that as
 * measured, so a plain fitView on switch runs straight away, skips every 0×0
 * node and leaves the viewport where it was — which is what happened on
 * opening a freshly created subworkflow. Waiting for real sizes avoids it.
 */
export default (currentWorkflowId: string) => {
  const { fitView } = useReactFlow();
  const store = useStoreApi();
  // Re-renders when nodes go from unmeasured to measured after a switch.
  const measured = useStore(allNodesMeasured);
  const pendingRef = useRef(false);

  useEffect(() => {
    pendingRef.current = true;
    const timeout = setTimeout(() => {
      if (!pendingRef.current) return;
      pendingRef.current = false;
      fitView({ padding: FIT_VIEW_PADDING });
    }, MEASURE_TIMEOUT);
    return () => clearTimeout(timeout);
  }, [currentWorkflowId, fitView]);

  useEffect(() => {
    // Read the store here, not the render-time value: on the switch render the
    // selector still sees the previous workflow's (measured) nodes.
    if (!pendingRef.current || !allNodesMeasured(store.getState())) return;
    pendingRef.current = false;
    fitView({ padding: FIT_VIEW_PADDING });
  }, [currentWorkflowId, measured, store, fitView]);
};
