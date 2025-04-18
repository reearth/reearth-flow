import {
  Copy,
  Eye,
  GearFine,
  Graph,
  Scissors,
  Trash,
} from "@phosphor-icons/react";
import { useCallback, useMemo } from "react";

import {
  ContextMenu,
  ContextMenuItemType,
  ContextMenuMeta,
  ContextMenuShortcut,
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
  onCopy?: () => void;
  onCut?: () => void;
  onClose: () => void;
};

const NodeContextMenu: React.FC<Props> = ({
  node,
  contextMenu,
  onNodesChange,
  onSecondaryNodeAction,
  onCopy,
  onCut,
  onClose,
}) => {
  const t = useT();
  const { id } = node;
  const handleNodeDelete = useCallback(() => {
    onNodesChange?.([{ id, type: "remove" }]);
  }, [id, onNodesChange]);

  const handleSecondaryNodeAction = useCallback(
    (allowNodeSettings?: boolean) => {
      if (!id) return;
      if (allowNodeSettings) {
        onSecondaryNodeAction?.(undefined, id, undefined);
        return;
      }

      onSecondaryNodeAction?.(undefined, id, node.data.subworkflowId);
    },
    [id, node.data.subworkflowId, onSecondaryNodeAction],
  );

  const menuItems = useMemo(() => {
    const wrapWithClose = (callback: () => void) => () => {
      callback();
      onClose();
    };

    const items: ContextMenuItemType[] = [
      {
        type: "action",
        props: {
          label: t("Copy"),
          icon: <Copy weight="light" />,
          shortcut: (
            <ContextMenuShortcut keyBinding={{ key: "c", commandKey: true }} />
          ),
          onCallback: wrapWithClose(onCopy ?? (() => {})),
        },
      },
      {
        type: "action",
        props: {
          label: t("Cut"),
          icon: <Scissors weight="light" />,
          shortcut: (
            <ContextMenuShortcut keyBinding={{ key: "x", commandKey: true }} />
          ),
          onCallback: wrapWithClose(onCut ?? (() => {})),
        },
      },
      ...(node.type === "subworkflow"
        ? [
            {
              type: "action" as const,
              props: {
                label: t("Open Subworkflow Canvas"),
                icon: <Graph weight="light" />,
                onCallback: wrapWithClose(handleSecondaryNodeAction),
              },
            },
          ]
        : []),
      {
        type: "action",
        props: {
          label: t("Node Settings"),
          icon: <GearFine weight="light" />,
          onCallback: wrapWithClose(() => handleSecondaryNodeAction(true)),
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
  }, [
    t,
    node,
    onCopy,
    onCut,
    handleSecondaryNodeAction,
    handleNodeDelete,
    onClose,
  ]);

  return <ContextMenu items={menuItems} contextMenuMeta={contextMenu} />;
};

export default NodeContextMenu;
