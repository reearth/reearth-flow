import { useReactFlow } from "@xyflow/react";
import { debounce } from "lodash-es";
import { useCallback, useEffect, useState } from "react";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
  Input,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@flow/components";
import { config } from "@flow/config";
import { useDoubleClick } from "@flow/hooks";
import { useAction } from "@flow/lib/fetch";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import { useT } from "@flow/lib/i18n";
import type { Action, ActionsSegregated, Node, Segregated } from "@flow/types";
import { randomID } from "@flow/utils";

import ActionComponent from "./Action";

type Ordering = "default" | "categorically" | "byType";

type Props = {
  nodes: Node[];
  onNodesChange: (nodes: Node[]) => void;
  onNodeLocking: (nodeId: string) => void;
};

const ActionsList: React.FC<Props> = ({
  nodes,
  onNodesChange,
  onNodeLocking,
}) => {
  const t = useT();
  const { useGetActions, useGetActionsSegregated } = useAction();

  const { screenToFlowPosition } = useReactFlow();

  const [selected, setSelected] = useState<string | undefined>(undefined);

  const [actions, setActions] = useState<Action[] | undefined>();

  const [actionsSegregated, setActionsSegregated] = useState<
    Segregated | undefined
  >();

  const { actions: actionsData } = useGetActions();
  const { actions: actionsSegregatedData } = useGetActionsSegregated();

  useEffect(() => {
    if (actionsData) setActions(actionsData);
    if (actionsSegregatedData) setActionsSegregated(actionsSegregatedData);
  }, [actionsData, actionsSegregatedData]);

  const tabs: {
    title: string;
    order: Ordering;
    actions: Action[] | ActionsSegregated | undefined;
  }[] = [
    {
      title: t("Alphabetical"),
      order: "default",
      actions,
    },
    {
      title: t("Category"),
      order: "categorically",
      actions: actionsSegregated?.byCategory,
    },
    {
      title: t("Type"),
      order: "byType",
      actions: actionsSegregated?.byType,
    },
  ];

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
        position: screenToFlowPosition({
          x: window.innerWidth / 2,
          y: window.innerHeight / 2,
        }),
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
    }
  );

  const handleActionSelect = (name?: string) => {
    setSelected((prevName) => (prevName === name ? undefined : name));
  };

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
      setActions(actionsData);
      setActionsSegregated(actionsSegregatedData);
      return;
    }

    const filteredActions =
      actionsData && getFilteredActions(filter, actionsData);
    setActions(filteredActions);

    const actionsSegregated =
      actionsSegregatedData &&
      Object.keys(actionsSegregatedData).reduce((obj, rootKey) => {
        obj[rootKey] = Object.keys(actionsSegregatedData[rootKey]).reduce(
          (obj: Record<string, Action[] | undefined>, key) => {
            obj[key] = getFilteredActions(
              filter,
              actionsSegregatedData[rootKey][key]
            );
            return obj;
          },
          {}
        );
        return obj;
      }, {} as Segregated);

    setActionsSegregated(actionsSegregated);
  }, 200);

  return (
    <Tabs defaultValue={tabs[0].order}>
      <div className="absolute w-full bg-background px-2">
        <TabsList className="flex justify-between">
          {tabs.map(({ title, order }) => (
            <TabsTrigger key={order} value={order} className="w-full">
              {title}
            </TabsTrigger>
          ))}
        </TabsList>
        <div>
          <Input
            className="mx-auto my-2 h-7 w-full"
            placeholder={t("Search")}
            onChange={(e) => handleSearch(e.target.value)}
          />
        </div>
      </div>
      <div className="mt-[52px] p-2">
        {tabs.map(({ order, actions }) => (
          <TabsContent
            className="dark flex flex-col gap-1"
            key={order}
            value={order}
          >
            {Array.isArray(actions) ? (
              actions.map((action) => (
                <ActionComponent
                  key={action.name}
                  action={action}
                  selected={selected === action.name}
                  onSingleClick={handleSingleClick}
                  onDoubleClick={handleDoubleClick}
                  onSelect={() => handleActionSelect(action.name)}
                />
              ))
            ) : (
              <Accordion type="single" collapsible>
                {actions ? (
                  Object.keys(actions).map((key) => (
                    <AccordionItem key={key} value={key}>
                      <AccordionTrigger>
                        <p className="capitalize">{key}</p>
                      </AccordionTrigger>
                      <AccordionContent>
                        {actions[key]?.map((action) => (
                          <ActionComponent
                            key={action.name}
                            action={action}
                            selected={selected === action.name}
                            onSingleClick={handleSingleClick}
                            onDoubleClick={handleDoubleClick}
                            onSelect={() => handleActionSelect(action.name)}
                          />
                        ))}
                      </AccordionContent>
                    </AccordionItem>
                  ))
                ) : (
                  <p className="mt-4 text-center">{t("Loading")}...</p>
                )}
              </Accordion>
            )}
          </TabsContent>
        ))}
      </div>
    </Tabs>
  );
};

export { ActionsList };
