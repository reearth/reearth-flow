import { Eye, GearFine, Graph, Trash } from "@phosphor-icons/react";
import { useReactFlow } from "@xyflow/react";
import { memo, useCallback, useMemo } from "react";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@flow/components";
import { useEditorContext } from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import { isActionNodeType, Node, NodeType } from "@flow/types";

type Props = {
  nodeId: string;
  nodeType: NodeType;
  children: React.ReactNode;
};

const NodeContextMenu: React.FC<Props> = ({
  nodeId,
  nodeType,
  children: nodeComponent,
}) => {
  const t = useT();

  const { getNode } = useReactFlow<Node>();

  const node = useMemo(() => getNode(nodeId), [getNode, nodeId]);

  const { onNodesChange, onSecondaryNodeAction } = useEditorContext();

  const handleNodeDelete = useCallback(() => {
    onNodesChange?.([{ id: nodeId, type: "remove" }]);
  }, [nodeId, onNodesChange]);

  const handleSecondaryNodeAction = useCallback(() => {
    if (!node) return;
    onSecondaryNodeAction?.(undefined, node);
  }, [node, onSecondaryNodeAction]);

  return (
    <ContextMenu>
      <ContextMenuTrigger>{nodeComponent}</ContextMenuTrigger>
      <ContextMenuContent>
        {nodeType === "subworkflow" ? (
          <ContextMenuItem
            className="justify-between gap-4 text-xs"
            onClick={handleSecondaryNodeAction}>
            {t("Open Subworkflow Canvas")}
            <Graph weight="light" />
          </ContextMenuItem>
        ) : (
          <ContextMenuItem
            className="justify-between gap-4 text-xs"
            onClick={handleSecondaryNodeAction}>
            {t("Node Settings")}
            <GearFine weight="light" />
          </ContextMenuItem>
        )}
        {isActionNodeType(nodeType) && (
          <ContextMenuItem className="justify-between gap-4 text-xs" disabled>
            {t("Preview Intermediate Data")}
            <Eye weight="light" />
          </ContextMenuItem>
        )}

        {/* <ContextMenuItem
          className="justify-between gap-4 text-xs"
          disabled={!selected}>
          {t("Subworkflow from Selection")}
          <Graph weight="light" />
        </ContextMenuItem> */}
        <ContextMenuSeparator />
        <ContextMenuItem
          className="justify-between gap-4 text-xs text-destructive"
          onClick={handleNodeDelete}>
          {t("Delete Node")}
          <Trash weight="light" />
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
};

export default memo(NodeContextMenu);
