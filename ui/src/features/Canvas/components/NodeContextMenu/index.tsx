import { Eye, GearFine, Graph, Trash } from "@phosphor-icons/react";
import { useT } from "@flow/lib/i18n";
import { useEditorContext } from "@flow/features/Editor/editorContext";
import { useCallback } from "react";
import { isActionNodeType, Node } from "@flow/types";
import CustomContextMenuItem from "../ContextMenuItem";
type Props = {
  node: Node;
  closeSelectionMenu: () => void;
};

const NodeContextMenu: React.FC<Props> = ({ closeSelectionMenu, node }) => {
  const t = useT();
  const { onNodesChange, onSecondaryNodeAction } = useEditorContext();
  const { id } = node;
  const handleNodeDelete = useCallback(() => {
    onNodesChange?.([{ id, type: "remove" }]);
  }, [id, onNodesChange]);

  const handleSecondaryNodeAction = useCallback(() => {
    if (!id) return;
    onSecondaryNodeAction?.(undefined, id, node.data.subworkflowId);
  }, [id, node.data.subworkflowId, onSecondaryNodeAction]);

  return (
    <div className="min-w-[160px] select-none rounded-md border bg-card p-1 text-popover-foreground shadow-md">
      {node.type === "subworkflow" ? (
        <CustomContextMenuItem
          label={t("Open Subworkflow Canvas")}
          icon={<Graph weight="light" />}
          onAction={handleSecondaryNodeAction}
          onClose={closeSelectionMenu}
        />
      ) : (
        <CustomContextMenuItem
          label={t("Node Settings")}
          icon={<GearFine weight="light" />}
          onAction={handleSecondaryNodeAction}
          onClose={closeSelectionMenu}
        />
      )}

      {isActionNodeType(node.type) && (
        <div
          className="flex items-center justify-between gap-4 rounded-sm px-2 py-1.5 text-xs hover:bg-accent"
          onClick={() => {
            handleSecondaryNodeAction;
            closeSelectionMenu();
          }}>
          <p> {t("Preview Intermediate Data")}</p>
          <Eye weight="light" />
        </div>
      )}
      <div className="-mx-1 my-1 h-px bg-border" />
      <CustomContextMenuItem
        label={t("Delete Node")}
        icon={<Trash weight="light" />}
        destructive
        onAction={handleNodeDelete}
        onClose={closeSelectionMenu}
      />
    </div>
  );
};

export default NodeContextMenu;
