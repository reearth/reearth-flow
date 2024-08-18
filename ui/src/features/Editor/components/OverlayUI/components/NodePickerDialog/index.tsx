import { XYPosition } from "@xyflow/react";
import { debounce } from "lodash-es";
import { Fragment, useCallback, useEffect, useState } from "react";

import { Dialog, DialogContent, DialogTitle, Input } from "@flow/components";
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
  const { actions: rawActions } = useGetActionsSegregated();

  const [actions, setActions] = useState<Action[] | undefined>();

  useEffect(() => {
    if (rawActions && openedActionType?.nodeType)
      setActions(rawActions?.byType[openedActionType.nodeType]);
  }, [rawActions, openedActionType.nodeType]);

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

  const getFilteredActions = useCallback(
    (filter: string, actions?: Action[]): Action[] | undefined =>
      actions?.filter((action) =>
        Object.values(action)
          .reduce(
            (result, value) =>
              (result += (
                Array.isArray(value) ? value.join() : value
              ).toLowerCase()),
            ""
          )
          .includes(filter.toLowerCase())
      ),
    []
  );

  // Don't worry too much about this implementation. It's only placeholder till we get an actual one using API
  const handleSearch = debounce((filter: string) => {
    if (!filter) {
      setActions(rawActions?.byType[openedActionType.nodeType]);
      return;
    }

    const filteredActions = getFilteredActions(
      filter,
      rawActions?.byType[openedActionType.nodeType]
    );
    setActions(filteredActions);
  }, 200);

  return (
    <Dialog open={!!openedActionType} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogTitle>{t("Choose action")}</DialogTitle>
        <Input
          className="mx-auto w-full rounded-none border-x-0 border-t-0 border-zinc-700 bg-zinc-800 focus-visible:ring-0"
          placeholder={t("Search")}
          onChange={(e) => handleSearch(e.target.value)}
        />
        <div className="max-h-[50vh] overflow-scroll">
          {actions?.map((action) => (
            <Fragment key={action.name}>
              <ActionItem
                className="m-1"
                action={action}
                selected={selected === action.name}
                onSingleClick={handleSingleClick}
                onDoubleClick={handleDoubleClick}
              />
              <div className="mx-1 border-b" />
            </Fragment>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default NodePickerDialog;
