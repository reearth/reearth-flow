import { XYPosition } from "@xyflow/react";
import { debounce } from "lodash-es";
import { Fragment, useCallback, useEffect, useRef, useState } from "react";

import { Dialog, DialogContent, DialogTitle, Input } from "@flow/components";
import ActionItem from "@flow/components/ActionItem";
import { useDoubleClick } from "@flow/hooks";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import type { Action, ActionNodeType, Node } from "@flow/types";

import useBatch from "../../../Canvas/useBatch";
import { useCreateNode } from "../../../Canvas/useCreateNode";

type Props = {
  openedActionType: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  nodes: Node[];
  onNodesChange: (nodes: Node[]) => void;
  onClose: () => void;
};

const NodePickerDialog: React.FC<Props> = ({
  openedActionType,
  nodes,
  onNodesChange,
  onClose,
}) => {
  const t = useT();
  const { useGetActionsSegregated } = useAction();
  const { actions: rawActions } = useGetActionsSegregated();
  const [actions, setActions] = useState<Action[] | undefined>();
  const [selectedIndex, setSelectedIndex] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLDivElement | null)[]>([]);
  const { handleNodeDropInBatch } = useBatch();
  useEffect(() => {
    if (rawActions && openedActionType?.nodeType)
      setActions(rawActions?.byType[openedActionType.nodeType]);
  }, [rawActions, openedActionType.nodeType]);

  const [selected, setSelected] = useState<string | undefined>(undefined);

  useEffect(() => {
    if (actions?.length) {
      setSelected(actions[selectedIndex]?.name);
      const selectedItem = itemRefs.current[selectedIndex];
      if (selectedItem && containerRef.current) {
        selectedItem.scrollIntoView({
          behavior: "smooth",
          block: "nearest",
        });
      }
    }
  }, [selectedIndex, actions]);

  const { createNode } = useCreateNode();

  const [handleSingleClick, handleDoubleClick] = useDoubleClick(
    (name?: string) => {
      setSelected((prevName) => (prevName === name ? undefined : name));
    },
    async (name?: string) => {
      if (!name) return;

      const newNode = await createNode({
        position: openedActionType.position,
        type: name,
      });

      if (!newNode) return;

      const newNodes = [...nodes, newNode];
      onNodesChange(handleNodeDropInBatch(newNode, newNodes));
      onClose();
    },
  );

  const getFilteredActions = useCallback(
    (filter: string, actions?: Action[]): Action[] | undefined =>
      actions?.filter((action) =>
        (
          Object.values(action).reduce(
            (result, value) =>
              (result += (
                Array.isArray(value)
                  ? value.join()
                  : typeof value === "string"
                    ? value
                    : ""
              ).toLowerCase()),
            "",
          ) as string
        ).includes(filter.toLowerCase()),
      ),
    [],
  );

  const handleSearch = debounce((filter: string) => {
    if (!filter) {
      setActions(rawActions?.byType[openedActionType.nodeType]);
      return;
    }

    const filteredActions = getFilteredActions(
      filter,
      rawActions?.byType[openedActionType.nodeType],
    );
    setActions(filteredActions);
  }, 200);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Enter") {
        e.preventDefault();
        handleDoubleClick(selected);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prevIndex) =>
          prevIndex === 0 ? prevIndex : prevIndex - 1,
        );
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prevIndex) =>
          prevIndex === (actions?.length || 1) - 1 ? prevIndex : prevIndex + 1,
        );
      }
    },
    [handleDoubleClick, selected, actions],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [actions, selected, handleKeyDown]);

  return (
    <Dialog open={!!openedActionType} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogTitle>{t("Choose action")}</DialogTitle>
        <Input
          className="mx-auto w-full rounded-none border-x-0 border-t-0 border-zinc-700 bg-secondary focus-visible:ring-0"
          placeholder={t("Search")}
          autoFocus
          onChange={(e) => handleSearch(e.target.value)}
        />
        <div ref={containerRef} className="max-h-[50vh] overflow-scroll">
          {actions?.map((action, idx) => (
            <Fragment key={action.name}>
              <ActionItem
                ref={(el) => (itemRefs.current[idx] = el)}
                className={"m-1"}
                action={action}
                selected={selected === action.name}
                onSingleClick={handleSingleClick}
                onDoubleClick={handleDoubleClick}
              />
              {idx !== actions.length - 1 && <div className="mx-1 border-b" />}
            </Fragment>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default NodePickerDialog;
