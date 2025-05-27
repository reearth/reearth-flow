import {
  Clipboard,
  Copy,
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
import { Node, NodeChange } from "@flow/types";

type Props = {
  contextMenu: ContextMenuMeta;
  data?: Node | Node[];
  selectedEdgeIds?: string[];
  onNodesChange?: (changes: NodeChange[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onWorkflowOpen?: (workflowId: string) => void;
  onNodeSettings?: (e: React.MouseEvent | undefined, nodeId: string) => void;
  onCopy?: (node?: Node) => void;
  onCut?: (isCutByShortCut?: boolean, node?: Node) => void;
  onPaste?: (menuPosition?: XYPosition) => void;
  onClose: () => void;
};

const CanvasContextMenu: React.FC<Props> = ({
  contextMenu,
  data,
  onWorkflowOpen,
  onNodeSettings,
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
    (node: Node) => {
      onNodeSettings?.(undefined, node.id);
    },
    [onNodeSettings],
  );

  const handleSubworkflowOpen = useCallback(
    (node: Node) => {
      if (!node.data?.subworkflowId) return;

      onWorkflowOpen?.(node.data.subworkflowId);
    },
    [onWorkflowOpen],
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
          disabled: (!nodes && !node) || !onCut,
          onCallback: wrapWithClose(() => onCopy?.(node) ?? (() => {})),
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
          disabled: (!nodes && !node) || !onCut,
          onCallback: wrapWithClose(() => onCut?.(false, node) ?? (() => {})),
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
          disabled: !value?.clipboard || !onPaste,
          onCallback: wrapWithClose(() => onPaste?.(contextMenu.mousePosition)),
        },
      },
      ...(node && node.type === "subworkflow"
        ? [
            {
              type: "action" as const,
              props: {
                label: t("Open Subworkflow"),
                icon: <Graph weight="light" />,
                onCallback: wrapWithClose(() => handleSubworkflowOpen(node)),
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
                  handleSecondaryNodeAction(node),
                ),
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
                disabled: !onNodesChange || !onEdgesChange,

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
    onNodesChange,
    onEdgesChange,
    contextMenu.mousePosition,
    value,
    handleNodeDelete,
    handleSecondaryNodeAction,
    handleSubworkflowOpen,
  ]);

  return <ContextMenu items={menuItems} contextMenuMeta={contextMenu} />;
};

export default CanvasContextMenu;
