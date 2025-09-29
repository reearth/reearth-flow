import {
  ClipboardIcon,
  CopyIcon,
  EyeIcon,
  EyeSlashIcon,
  GearFineIcon,
  GraphIcon,
  ScissorsIcon,
  TrashIcon,
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
  allNodes: Node[];
  selectedEdgeIds?: string[];
  onNodesChange?: (changes: NodeChange[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onBeforeDelete?: (args: { nodes: Node[] }) => Promise<boolean>;
  onWorkflowOpen?: (workflowId: string) => void;
  onNodeSettings?: (e: React.MouseEvent | undefined, nodeId: string) => void;
  onCopy?: (node?: Node) => void;
  onCut?: (isCutByShortCut?: boolean, node?: Node) => void;
  onPaste?: (menuPosition?: XYPosition) => void;
  onNodeDisable?: (node?: Node) => void;
  onClose: () => void;
};

const CanvasContextMenu: React.FC<Props> = ({
  contextMenu,
  data,
  allNodes,
  onWorkflowOpen,
  onNodeSettings,
  selectedEdgeIds,
  onNodesChange,
  onEdgesChange,
  onBeforeDelete,
  onCopy,
  onCut,
  onPaste,
  onNodeDisable,
  onClose,
}) => {
  const t = useT();
  const { value } = useIndexedDB("general");

  const freshData = useMemo(() => {
    if (!data) return undefined;

    if (Array.isArray(data)) {
      const nodeIds = data.map((n) => n.id);
      return allNodes.filter((n) => nodeIds.includes(n.id));
    } else {
      return allNodes.find((n) => n.id === data.id);
    }
  }, [data, allNodes]);

  const nodes = Array.isArray(freshData) ? freshData : undefined;
  const node = Array.isArray(freshData) ? undefined : freshData;

  const handleNodeSettingsOpen = useCallback(
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
    async (node?: Node, nodes?: Node[]) => {
      if (!nodes && !node) return;

      const toDelete = nodes ?? (node ? [node] : []);
      const shouldDelete = await onBeforeDelete?.({ nodes: toDelete });

      if (shouldDelete) {
        onNodesChange?.(
          toDelete.map((node) => ({ id: node.id, type: "remove" as const })),
        );

        selectedEdgeIds?.forEach((edgeId) => {
          onEdgesChange?.([{ id: edgeId, type: "remove" as const }]);
        });
      }
    },
    [selectedEdgeIds, onBeforeDelete, onNodesChange, onEdgesChange],
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
          icon: <CopyIcon weight="light" />,
          shortcut: (
            <ContextMenuShortcut keyBinding={{ key: "c", commandKey: true }} />
          ),
          disabled: (!nodes && !node) || !onCopy,
          onCallback: wrapWithClose(() => onCopy?.(node) ?? (() => {})),
        },
      },
      {
        type: "action",
        props: {
          label: t("Cut"),
          icon: <ScissorsIcon weight="light" />,
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
          icon: <ClipboardIcon weight="light" />,
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
                icon: <GraphIcon weight="light" />,
                onCallback: wrapWithClose(() => handleSubworkflowOpen(node)),
              },
            },
          ]
        : []),
      {
        type: "action",
        props: {
          label: (() => {
            const selectedNodes = node
              ? [node]
              : nodes?.filter((n) => n.selected) || [];
            const anyEnabled = selectedNodes.some((n) => !n.data?.isDisabled);
            return anyEnabled ? t("Disable Node") : t("Enable Node");
          })(),
          icon: (() => {
            const selectedNodes = node
              ? [node]
              : nodes?.filter((n) => n.selected) || [];
            const anyEnabled = selectedNodes.some((n) => !n.data?.isDisabled);
            return anyEnabled ? (
              <EyeSlashIcon weight="light" />
            ) : (
              <EyeIcon weight="light" />
            );
          })(),
          shortcut: (
            <ContextMenuShortcut keyBinding={{ key: "e", commandKey: true }} />
          ),
          disabled: (!nodes && !node) || !onNodeDisable,
          onCallback: wrapWithClose(() => onNodeDisable?.(node) ?? (() => {})),
        },
      },
      ...(node
        ? [
            {
              type: "action" as const,
              props: {
                label: t("Node Settings"),
                icon: <GearFineIcon weight="light" />,
                onCallback: wrapWithClose(() => handleNodeSettingsOpen(node)),
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
                icon: <TrashIcon weight="light" />,
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
    onNodeDisable,
    contextMenu.mousePosition,
    value,
    handleNodeDelete,
    handleNodeSettingsOpen,
    handleSubworkflowOpen,
  ]);

  return <ContextMenu items={menuItems} contextMenuMeta={contextMenu} />;
};

export default CanvasContextMenu;
