import { Eye, GearFine, Graph, Trash } from "@phosphor-icons/react";
import { useCallback } from "react";

import { ContextMenu, MenuPosition } from "@flow/components";
import { useEditorContext } from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import { isActionNodeType, Node } from "@flow/types";

type Props = {
  node: Node;
  menuPosition: MenuPosition;
  onClose: () => void;
};

const NodeContextMenu: React.FC<Props> = ({ node, menuPosition, onClose }) => {
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

  const menuItems = [
    node.type === "subworkflow"
      ? {
          label: t("Open Subworkflow Canvas"),
          icon: <Graph weight="light" />,
          onCallback: handleSecondaryNodeAction,
          onClose,
        }
      : {
          label: t("Node Settings"),
          icon: <GearFine weight="light" />,
          onCallback: handleSecondaryNodeAction,
          onClose,
        },
    ...(isActionNodeType(node.type)
      ? [
          {
            label: t("Preview Intermediate Data"),
            icon: <Eye weight="light" />,
            onCallback: handleSecondaryNodeAction,
            disabled: true,
            onClose,
          },
        ]
      : []),
    {
      label: t("Delete Node"),
      icon: <Trash weight="light" />,
      destructive: true,
      onCallback: handleNodeDelete,
      onClose,
    },
  ];

  return (
    <ContextMenu
      items={menuItems}
      menuPosition={menuPosition}
      onClose={onClose}
    />
  );
};

export default NodeContextMenu;
