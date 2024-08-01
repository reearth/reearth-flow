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
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import { Action, ActionsSegregated, Segregated } from "@flow/types";

import { ActionComponent } from "./SingleAction";

type ActionTab = "All" | "Category" | "Type";

const ActionsList: React.FC = () => {
  const t = useT();
  const { useGetActions, useGetActionSegregated } = useAction();

  const [actions, setActions] = useState<Action[] | undefined>();

  const [actionsSegregated, setActionsSegregated] = useState<Segregated | undefined>();

  const { actions: actionsData } = useGetActions();
  const { actions: actionsSegregatedData } = useGetActionSegregated();

  useEffect(() => {
    if (actionsData) setActions(actionsData);
    if (actionsSegregatedData) setActionsSegregated(actionsSegregatedData);
  }, [actionsData, actionsSegregatedData]);

  const tabs: {
    title: string;
    value: ActionTab;
    actions: Action[] | ActionsSegregated | undefined;
  }[] = [
    {
      title: t("All"),
      value: "All",
      actions: actions,
    },
    {
      title: t("Category"),
      value: "Category",
      actions: actionsSegregated?.byCategory,
    },
    {
      title: t("Type"),
      value: "Type",
      actions: actionsSegregated?.byType,
    },
  ];

  const getFilteredActions = useCallback(
    (filter: string, actions?: Action[]): Action[] | undefined =>
      actions?.filter(action =>
        Object.values(action)
          .reduce(
            (result, value) =>
              (result += (Array.isArray(value) ? value.join() : value).toLowerCase()),
            "",
          )
          .includes(filter.toLowerCase()),
      ),
    [],
  );

  // Don't worry too much about this implementation. It's only placeholder till we get an actual one using API
  const handleSearch = debounce((filter: string) => {
    if (!filter) {
      setActions(actionsData);
      setActionsSegregated(actionsSegregatedData);
      return;
    }

    const filteredActions = actionsData && getFilteredActions(filter, actionsData);
    setActions(filteredActions);

    const filteredActionsSegregated =
      actionsSegregatedData &&
      Object.keys(actionsSegregatedData).reduce((obj, rootKey) => {
        obj[rootKey] = Object.keys(actionsSegregatedData[rootKey]).reduce(
          (obj: Record<string, Action[] | undefined>, key) => {
            obj[key] = getFilteredActions(filter, actionsSegregatedData[rootKey][key]);
            return obj;
          },
          {},
        );
        return obj;
      }, {} as Segregated);

    setActionsSegregated(filteredActionsSegregated);
  }, 200);

  return (
    <Tabs defaultValue={tabs[0].value}>
      <div className="absolute w-full bg-secondary p-2">
        <TabsList className="flex justify-between px-0">
          {tabs.map(({ title, value }) => (
            <TabsTrigger key={value} value={value} className="w-[31%] uppercase">
              {title}
            </TabsTrigger>
          ))}
        </TabsList>
        <div>
          <Input
            className="mx-auto mt-2 w-full px-2"
            placeholder={t("Search")}
            // value={search}
            onChange={e => handleSearch(e.target.value)}
          />
        </div>
      </div>
      <div className="mt-20 p-2">
        {tabs.map(({ value, actions }) => (
          <TabsContent className="dark" key={value} value={value}>
            {Array.isArray(actions) ? (
              actions.map(action => <ActionComponent key={action.name} {...action} />)
            ) : (
              <Accordion type="single" collapsible>
                {actions ? (
                  Object.keys(actions).map(key => (
                    <AccordionItem key={key} value={key}>
                      <AccordionTrigger>{key}</AccordionTrigger>
                      <AccordionContent>
                        {actions[key]?.map(action => (
                          <ActionComponent key={action.name} {...action} />
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
