import { Trash } from "@phosphor-icons/react";
import { EdgeChange } from "@xyflow/react";
import { useCallback, useMemo } from "react";

import { ContextMenu, ContextMenuMeta } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Node, NodeChange } from "@flow/types";

type Props = {
  nodes: Node[];
  selectedEdgeIds?: string[];
  contextMenu: ContextMenuMeta;
  onNodesChange: (changes: NodeChange[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onClose: () => void;
};

const SelectionContextMenu: React.FC<Props> = ({
  nodes,
  contextMenu,
  selectedEdgeIds,
  onNodesChange,
  onEdgesChange,
  onClose,
}) => {
  const t = useT();

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

    const items = [
      {
        label: t("Delete Selection"),
        icon: <Trash weight="light" />,
        destructive: true,
        onCallback: wrapWithClose(handleNodeDelete),
      },
    ];

    return items;
  }, [t, handleNodeDelete, onClose]);

  return <ContextMenu items={menuItems} contextMenuMeta={contextMenu} />;
};

export default SelectionContextMenu;
