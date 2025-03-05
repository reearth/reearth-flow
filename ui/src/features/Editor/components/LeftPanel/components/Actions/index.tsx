import { useReactFlow } from "@xyflow/react";
import { debounce } from "lodash-es";
import { Fragment, useEffect, useState } from "react";

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
import i18n from "@flow/lib/i18n/i18n";
import type { Action, ActionsSegregated, Node, NodeType } from "@flow/types";
import { generateUUID } from "@flow/utils";
import { getRandomNumberInRange } from "@flow/utils/getRandomNumberInRange";

import ActionComponent from "./Action";

type Ordering = "default" | "categorically" | "byType";

type Props = {
  nodes: Node[];
  onNodesAdd: (nodes: Node[]) => void;
  isMainWorkflow: boolean;
  hasReader?: boolean;
};

const ActionsList: React.FC<Props> = ({
  nodes,
  onNodesAdd,
  isMainWorkflow,
  hasReader,
}) => {
  const t = useT();
  const { useGetActions, useGetActionsSegregated } = useAction(i18n.language);
  const { screenToFlowPosition } = useReactFlow();
  const [selected, setSelected] = useState<string | undefined>(undefined);
  const [searchTerm, setSearchTerm] = useState<string>("");
  const [searchDone, setSearchDone] = useState<string>("");

  const { actions } = useGetActions({
    isMainWorkflow,
    searchTerm: searchDone,
  });

  const { actions: actionsSegregated } = useGetActionsSegregated({
    isMainWorkflow,
    searchTerm: searchDone,
    nodes,
  });

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
      const randomX = getRandomNumberInRange(50, 200);
      const randomY = getRandomNumberInRange(50, 200);
      const newNode: Node = {
        id: generateUUID(),
        type: action.type as NodeType,
        position: screenToFlowPosition({
          x: window.innerWidth / 2 + randomX,
          y: window.innerHeight / 2 - randomY,
        }),
        data: {
          officialName: action.name,
          inputs: [...action.inputPorts],
          outputs: [...action.outputPorts],
        },
      };
      onNodesAdd([newNode]);
    },
  );

  const handleActionSelect = (name?: string) => {
    setSelected((prevName) => (prevName === name ? undefined : name));
  };

  const handleSearch = debounce((filter: string) => {
    setSearchDone(filter);
  }, 200);

  useEffect(() => {
    if (searchTerm !== searchDone) {
      handleSearch(searchTerm);
    }
  }, [searchTerm, searchDone, handleSearch]);

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
            value={searchTerm}
            onChange={(e) => {
              setSearchTerm(e.target.value);
            }}
          />
        </div>
      </div>
      <div className="mt-[52px] p-2">
        {tabs.map(({ order, actions }) => (
          <TabsContent
            className="flex flex-col gap-1"
            key={order}
            value={order}>
            {Array.isArray(actions) ? (
              actions?.map((action, index) => {
                if (action.type === "reader" && hasReader) return null;
                return (
                  <Fragment key={action.name}>
                    <ActionComponent
                      action={action}
                      selected={selected === action.name}
                      onTypeClick={(type) =>
                        setSearchTerm((st) => (st === type ? "" : type))
                      }
                      onCategoryClick={(category) =>
                        setSearchTerm((st) => (st === category ? "" : category))
                      }
                      onSingleClick={handleSingleClick}
                      onDoubleClick={handleDoubleClick}
                      onSelect={() => handleActionSelect(action.name)}
                    />
                    {index !== actions.length - 1 && (
                      <div className="border-b" />
                    )}
                  </Fragment>
                );
              })
            ) : (
              <Accordion type="single" collapsible>
                {actions ? (
                  Object.entries(actions)
                    .filter(([key]) => {
                      if (isMainWorkflow) {
                        if (key === "reader" && hasReader) return false;
                        return true;
                      } else {
                        return key !== "reader" && key !== "writer";
                      }
                    })
                    .map(([key, categoryActions]) => (
                      <AccordionItem key={key} value={key}>
                        <AccordionTrigger>
                          <p className="capitalize">{key}</p>
                        </AccordionTrigger>
                        <AccordionContent className="flex flex-col gap-1">
                          {categoryActions?.map((action, index) => {
                            if (action.type === "reader" && hasReader)
                              return null;
                            return (
                              <Fragment key={action.name}>
                                <ActionComponent
                                  action={action}
                                  selected={selected === action.name}
                                  onTypeClick={(type) =>
                                    setSearchTerm((st) =>
                                      st === type ? "" : type,
                                    )
                                  }
                                  onCategoryClick={(category) =>
                                    setSearchTerm((st) =>
                                      st === category ? "" : category,
                                    )
                                  }
                                  onSingleClick={handleSingleClick}
                                  onDoubleClick={handleDoubleClick}
                                  onSelect={() =>
                                    handleActionSelect(action.name)
                                  }
                                />
                                {categoryActions &&
                                  index !== categoryActions.length - 1 && (
                                    <div className="border-b" />
                                  )}
                              </Fragment>
                            );
                          })}
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
