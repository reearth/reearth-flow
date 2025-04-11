import { Clipboard, Copy, Scissors, Trash } from "@phosphor-icons/react";
import { EdgeChange } from "@xyflow/react";
import { useCallback, useMemo } from "react";

import {
  ContextMenu,
  ContextMenuItemType,
  ContextMenuMeta,
  ContextMenuShortcut,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import type { Node, NodeChange } from "@flow/types";

type Props = {
  nodes: Node[];
  selectedEdgeIds?: string[];
  contextMenu: ContextMenuMeta;
  onNodesChange: (changes: NodeChange[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onCopy?: () => void;
  onCut?: () => void;
  onPaste?: () => void;
  onClose: () => void;
};

const SelectionContextMenu: React.FC<Props> = ({
  nodes,
  contextMenu,
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

  const handleNodeDelete = useCallback(() => {
    nodes.forEach((node) => {
      onNodesChange?.([{ id: node.id, type: "remove" as const }]);
    });
    selectedEdgeIds?.forEach((edgeId) => {
      onEdgesChange?.([{ id: edgeId, type: "remove" as const }]);
    });
  }, [nodes, selectedEdgeIds, onNodesChange, onEdgesChange]);

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
          onCallback: wrapWithClose(onPaste ?? (() => {})),
        },
      },
      {
        type: "action",
        props: {
          label: t("Delete Selection"),
          icon: <Trash weight="light" />,
          destructive: true,
          onCallback: wrapWithClose(handleNodeDelete),
        },
      },
    ];

    return items;
  }, [t, handleNodeDelete, onCopy, onCut, onClose, onPaste, value]);

  return <ContextMenu items={menuItems} contextMenuMeta={contextMenu} />;
};

export default SelectionContextMenu;
