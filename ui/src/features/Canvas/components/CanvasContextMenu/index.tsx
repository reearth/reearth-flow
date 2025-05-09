import {
  Clipboard,
  Copy,
  Eye,
  GearFine,
  Graph,
  Scissors,
  Trash,
} from "@phosphor-icons/react";
import { EdgeChange, XYPosition } from "@xyflow/react";
import { useCallback, useMemo } from "react";

import {
  ContextMenu,
  ContextMenuItemType,
  ContextMenuMeta,
  ContextMenuShortcut,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { isActionNodeType, Node, NodeChange } from "@flow/types";

type Props = {
  contextMenu: ContextMenuMeta;
  data?: Node | Node[];
  selectedEdgeIds?: string[];
  onNodesChange?: (changes: NodeChange[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onSecondaryNodeAction?: (
    e: React.MouseEvent | undefined,
    nodeId: string,
    subworkflowId?: string,
  ) => void;
  onCopy?: () => void;
  onCut?: () => void;
  onPaste?: (menuPosition?: XYPosition) => void;
  onClose: () => void;
};

const CanvasContextMenu: React.FC<Props> = ({
  contextMenu,
  data,
  onSecondaryNodeAction,
  selectedEdgeIds,
  onNodesChange,
  onEdgesChange,
  onCopy,
  onCut,
  onPaste,
  onClose,
}) => {
  const t = useT();
  const { value } = useIndexedDB("general");

  const nodes = Array.isArray(data) ? data : undefined;
  const node = Array.isArray(data) ? undefined : data;

  const handleSecondaryNodeAction = useCallback(
    (node?: Node, allowNodeSettings?: boolean) => {
      if (!node) return;

      if (allowNodeSettings) {
        onSecondaryNodeAction?.(undefined, node.id, undefined);

        return;
      }

      onSecondaryNodeAction?.(undefined, node.id, node.data.subworkflowId);
    },
    [onSecondaryNodeAction],
  );

  const handleNodeDelete = useCallback(
    (node?: Node, nodes?: Node[]) => {
      if (!nodes && !node) return;

      if (nodes) {
        nodes.forEach((node) => {
          onNodesChange?.([{ id: node.id, type: "remove" as const }]);
        });
        selectedEdgeIds?.forEach((edgeId) => {
          onEdgesChange?.([{ id: edgeId, type: "remove" as const }]);
        });
      } else if (node) {
        onNodesChange?.([{ id: node.id, type: "remove" }]);
      }
    },
    [selectedEdgeIds, onNodesChange, onEdgesChange],
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
          disabled: !nodes && !node,
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
          disabled: !nodes && !node,
          onCallback: wrapWithClose(onCut ?? (() => {})),
        },
      },
      {
        type: "action",
        props: {
          label: t("Paste"),
          icon: <Clipboard weight="light" />,
          shortcut: (
            <ContextMenuShortcut keyBinding={{ key: "v", commandKey: true }} />
          ),
          disabled: !value?.clipboard,
          onCallback: wrapWithClose(() => onPaste?.(contextMenu.mousePosition)),
        },
      },
      ...(node && node.type === "subworkflow"
        ? [
            {
              type: "action" as const,
              props: {
                label: t("Open Subworkflow Canvas"),
                icon: <Graph weight="light" />,
                onCallback: wrapWithClose(() =>
                  handleSecondaryNodeAction(node),
                ),
              },
            },
          ]
        : []),
      ...(node
        ? [
            {
              type: "action" as const,
              props: {
                label: t("Node Settings"),
                icon: <GearFine weight="light" />,
                onCallback: wrapWithClose(() =>
                  handleSecondaryNodeAction(node, true),
                ),
              },
            },
          ]
        : []),
      ...(node && isActionNodeType(node.type)
        ? [
            {
              type: "action" as const,
              props: {
                label: t("Preview Intermediate Data"),
                icon: <Eye weight="light" />,
                onCallback: wrapWithClose(() =>
                  handleSecondaryNodeAction(node),
                ),
                disabled: true,
              },
            },
          ]
        : []),
      ...(node || nodes
        ? [
            {
              type: "separator" as const,
            },
          ]
        : []),

      ...(node || nodes
        ? [
            {
              type: "action" as const,
              props: {
                label: node ? t("Delete Node") : t("Delete Selection"),
                icon: <Trash weight="light" />,
                destructive: true,
                onCallback: wrapWithClose(() => handleNodeDelete(node, nodes)),
              },
            },
          ]
        : []),
    ];

    return items;
  }, [
    t,
    node,
    nodes,
    onCopy,
    onCut,
    onPaste,
    onClose,
    contextMenu.mousePosition,
    value,
    handleNodeDelete,
    handleSecondaryNodeAction,
  ]);

  return <ContextMenu items={menuItems} contextMenuMeta={contextMenu} />;
};

export default CanvasContextMenu;
