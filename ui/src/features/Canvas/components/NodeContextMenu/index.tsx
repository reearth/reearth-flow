import { Eye, GearFine, Graph, Trash } from "@phosphor-icons/react";
import { useCallback, useMemo } from "react";

import { ContextMenu, ContextMenuMeta } from "@flow/components";
import { useEditorContext } from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import { isActionNodeType, Node } from "@flow/types";

type Props = {
  node: Node;
  contextMenu: ContextMenuMeta;
  onClose: () => void;
};

const NodeContextMenu: React.FC<Props> = ({ node, contextMenu, onClose }) => {
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

  const menuItems = useMemo(() => {
    const wrapWithClose = (callback: () => void) => () => {
      callback();
      onClose();
    };

    const items = [
      node.type === "subworkflow"
        ? {
            label: t("Open Subworkflow Canvas"),
            icon: <Graph weight="light" />,
            onCallback: wrapWithClose(handleSecondaryNodeAction),
          }
        : {
            label: t("Node Settings"),
            icon: <GearFine weight="light" />,
            onCallback: wrapWithClose(handleSecondaryNodeAction),
          },
      ...(isActionNodeType(node.type)
        ? [
            {
              label: t("Preview Intermediate Data"),
              icon: <Eye weight="light" />,
              onCallback: wrapWithClose(handleSecondaryNodeAction),
              disabled: true,
            },
          ]
        : []),
      {
        label: t("Delete Node"),
        icon: <Trash weight="light" />,
        destructive: true,
        onCallback: wrapWithClose(handleNodeDelete),
      },
    ];

    return items;
  }, [t, node, handleSecondaryNodeAction, handleNodeDelete, onClose]);

  return <ContextMenu items={menuItems} contextMenuMeta={contextMenu} />;
};

export default NodeContextMenu;
