import { XYPosition } from "@xyflow/react";
import { useState } from "react";

import { Dialog, DialogContent, DialogTitle } from "@flow/components";
import ActionItem from "@flow/components/ActionItem";
import { config } from "@flow/config";
import { useDoubleClick } from "@flow/hooks";
import { useAction } from "@flow/lib/fetch";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import { useT } from "@flow/lib/i18n";
import type { Action, ActionNodeType, Node } from "@flow/types";
import { randomID } from "@flow/utils";

type Props = {
  openedActionType: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  nodes: Node[];
  onNodesChange: (nodes: Node[]) => void;
  onNodeLocking: (nodeId: string) => void;
  onClose: () => void;
};

const NodePickerDialog: React.FC<Props> = ({
  openedActionType,
  nodes,
  onNodesChange,
  onNodeLocking,
  onClose,
}) => {
  const t = useT();
  const { useGetActionsSegregated } = useAction();
  const { actions } = useGetActionsSegregated();

  const [selected, setSelected] = useState<string | undefined>(undefined);

  const [handleSingleClick, handleDoubleClick] = useDoubleClick(
    (name?: string) => {
      setSelected((prevName) => (prevName === name ? undefined : name));
    },
    async (name?: string) => {
      const { api } = config();
      const action = await fetcher<Action>(`${api}/actions/${name}`);
      if (!action) return;

      const newNode: Node = {
        id: randomID(),
        type: action.type,
        position: openedActionType.position,
        data: {
          name: action.name,
          inputs: [...action.inputPorts],
          outputs: [...action.outputPorts],
          status: "idle",
          locked: false,
          onDoubleClick: onNodeLocking,
        },
      };
      onNodesChange(nodes.concat(newNode));
      onClose();
    }
  );

  return (
    <Dialog open={!!openedActionType} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogTitle>{t("Node Picker")}</DialogTitle>
        <div className="h-[30vh] overflow-scroll">
          {actions?.byType[openedActionType.nodeType]?.map((action) => (
            <ActionItem
              key={action.name}
              action={action}
              selected={selected === action.name}
              onSingleClick={handleSingleClick}
              onDoubleClick={handleDoubleClick}
            />
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default NodePickerDialog;
