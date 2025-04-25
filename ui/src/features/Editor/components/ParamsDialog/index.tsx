import { useReactFlow } from "@xyflow/react";
import { memo, useCallback, useEffect, useRef } from "react";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Node } from "@flow/types";

import { ParamEditor } from "./components";

type Props = {
  selected?: Node;
  onDataSubmit?: (
    nodeId: string,
    dataField: "params" | "customizations",
    updatedValue: any,
  ) => void;
  onWorkflowRename?: (id: string, name: string) => void;
};

const ParamsDialog: React.FC<Props> = ({
  selected,
  onDataSubmit,
  onWorkflowRename,
}) => {
  const t = useT();
  // This is a little hacky, but it works. We need to dispatch a click event to the react-flow__pane
  // to unlock the node when user wants to close the right panel. - @KaWaite
  const handleClose = useCallback(() => {
    // react-flow__pane is the classname of the div inside react-flow that has the click event
    // https://github.com/xyflow/xyflow/blob/71db83761c245493d44e74311e10cc6465bf8387/packages/react/src/container/Pane/index.tsx#L249
    const paneElement = document.getElementsByClassName("react-flow__pane")[0];
    if (!paneElement) return;
    const clickEvent = new Event("click", { bubbles: true, cancelable: true });
    paneElement.dispatchEvent(clickEvent);
  }, []);

  const handleUpdate = useCallback(
    async (nodeId: string, data: any, type: "params" | "customizations") => {
      if (type === "params") {
        await Promise.resolve(onDataSubmit?.(nodeId, "params", data));
      } else if (type === "customizations") {
        await Promise.resolve(onDataSubmit?.(nodeId, "customizations", data));
      }
      handleClose();
    },
    [onDataSubmit, handleClose],
  );

  const { getViewport, setViewport } = useReactFlow();

  const previousViewportRef = useRef<{
    x: number;
    y: number;
    zoom: number;
  } | null>(null);

  useEffect(() => {
    if (selected && !previousViewportRef.current) {
      const { x, y, zoom } = getViewport();
      previousViewportRef.current = { x, y, zoom };
    } else if (!selected && previousViewportRef.current) {
      setViewport(previousViewportRef.current, { duration: 400 });
      previousViewportRef.current = null;
    }
  }, [setViewport, getViewport, selected]);

  return (
    <Dialog open={!!selected} onOpenChange={handleClose}>
      <DialogContent size="2xl">
        <DialogHeader>
          <DialogTitle>{t("Parameter Editor")}</DialogTitle>
        </DialogHeader>
        {selected && (
          <ParamEditor
            nodeId={selected.id}
            nodeMeta={selected.data}
            nodeType={selected.type}
            onUpdate={handleUpdate}
            onWorkflowRename={onWorkflowRename}
          />
        )}
      </DialogContent>
    </Dialog>
  );
};

export default memo(ParamsDialog);
