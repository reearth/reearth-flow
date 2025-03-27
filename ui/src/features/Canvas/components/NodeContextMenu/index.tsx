import { Eye, GearFine, Graph, Trash } from "@phosphor-icons/react";
import { useCallback, useMemo } from "react";

import {
  ContextMenu,
  ContextMenuItemType,
  ContextMenuMeta,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { isActionNodeType, Node, NodeChange } from "@flow/types";

type Props = {
  node: Node;
  contextMenu: ContextMenuMeta;
  onNodesChange: (changes: NodeChange[]) => void;
  onSecondaryNodeAction?: (
    e: React.MouseEvent | undefined,
    nodeId: string,
    subworkflowId?: string,
  ) => void;
  onClose: () => void;
};

const NodeContextMenu: React.FC<Props> = ({
  node,
  contextMenu,
  onNodesChange,
  onSecondaryNodeAction,
  onClose,
}) => {
  const t = useT();
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

    const items: ContextMenuItemType[] = [
      node.type === "subworkflow"
        ? {
            type: "action",
            props: {
              label: t("Open Subworkflow Canvas"),
              icon: <Graph weight="light" />,
              onCallback: wrapWithClose(handleSecondaryNodeAction),
            },
          }
        : {
            type: "action",
            props: {
              label: t("Node Settings"),
              icon: <GearFine weight="light" />,
              onCallback: wrapWithClose(handleSecondaryNodeAction),
            },
          },
      ...(isActionNodeType(node.type)
        ? [
            {
              type: "action" as const,
              props: {
                label: t("Preview Intermediate Data"),
                icon: <Eye weight="light" />,
                onCallback: wrapWithClose(handleSecondaryNodeAction),
                disabled: true,
              },
            },
          ]
        : []),
      {
        type: "separator",
      },
      {
        type: "action",
        props: {
          label: t("Delete Node"),
          icon: <Trash weight="light" />,
          destructive: true,
          onCallback: wrapWithClose(handleNodeDelete),
        },
      },
    ];

    return items;
  }, [t, node, handleSecondaryNodeAction, handleNodeDelete, onClose]);

  return <ContextMenu items={menuItems} contextMenuMeta={contextMenu} />;
};

export default NodeContextMenu;
