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
  readonly?: boolean;
  openNode?: Node;
  onOpenNode: (nodeId?: string) => void;
  onDataSubmit?: (
    nodeId: string,
    dataField: "params" | "customizations",
    updatedValue: any,
  ) => void;
  onWorkflowRename?: (id: string, name: string) => void;
};

const ParamsPanel: React.FC<Props> = ({
  openNode,
  onOpenNode,
  readonly,
  onDataSubmit,
  onWorkflowRename,
}) => {
  const t = useT();

  const handleUpdate = useCallback(
    async (nodeId: string, data: any, type: "params" | "customizations") => {
      if (type === "params") {
        await Promise.resolve(onDataSubmit?.(nodeId, "params", data));
      } else if (type === "customizations") {
        await Promise.resolve(onDataSubmit?.(nodeId, "customizations", data));
      }
      onOpenNode();
    },
    [onDataSubmit, onOpenNode],
  );

  const { getViewport, setViewport } = useReactFlow();

  const previousViewportRef = useRef<{
    x: number;
    y: number;
    zoom: number;
  } | null>(null);

  useEffect(() => {
    if (openNode && !previousViewportRef.current) {
      const { x, y, zoom } = getViewport();
      previousViewportRef.current = { x, y, zoom };
    } else if (!openNode && previousViewportRef.current) {
      setViewport(previousViewportRef.current, { duration: 400 });
      previousViewportRef.current = null;
    }
  }, [setViewport, getViewport, openNode]);

  return (
    <Dialog open={!!openNode} onOpenChange={() => onOpenNode()}>
      <DialogContent size="2xl">
        <DialogHeader>
          <DialogTitle>{t("Parameter Editor")}</DialogTitle>
        </DialogHeader>
        {openNode && (
          <ParamEditor
            readonly={readonly}
            nodeId={openNode.id}
            nodeMeta={openNode.data}
            nodeType={openNode.type}
            onUpdate={handleUpdate}
            onWorkflowRename={onWorkflowRename}
          />
        )}
      </DialogContent>
    </Dialog>
  );
};

export default memo(ParamsPanel);
