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
import { useT } from "@flow/lib/i18n";
import actions from "@flow/mock_data/actions";
import actionsSegregated from "@flow/mock_data/actionsSegregated";
import { Action, Segregated } from "@flow/types";

import { ActionComponent } from "./SingleAction";

type ActionTab = "All" | "Category" | "Type";

const ActionsList: React.FC = () => {
  const t = useT();

  const tabs: { title: string; value: ActionTab; actions: Action[] | Segregated }[] = [
    {
      title: t("All"),
      value: "All",
      actions: actions,
    },
    {
      title: t("Category"),
      value: "Category",
      actions: actionsSegregated.byCategory,
    },
    {
      title: t("Type"),
      value: "Type",
      actions: actionsSegregated.byType,
    },
  ];

  return (
    <Tabs defaultValue={tabs[1].value}>
      <TabsList className="flex justify-between px-2 *:w-[31%]">
        {tabs.map(({ title, value }) => (
          <TabsTrigger key={value} value={value} className="uppercase">
            {title}
          </TabsTrigger>
        ))}
      </TabsList>
      <div className="px-2">
        <Input className="mx-auto mt-2 w-full px-2" placeholder={t("Search")} disabled />
      </div>
      <div className="p-2">
        {tabs.map(({ value, actions }) => (
          <TabsContent className="dark" key={value} value={value}>
            {Array.isArray(actions) ? (
              actions.map(action => <ActionComponent key={action.name} {...action} />)
            ) : (
              <Accordion type="single" collapsible>
                {Object.keys(actions).map(key => (
                  <AccordionItem key={key} value={key}>
                    <AccordionTrigger>{key}</AccordionTrigger>
                    <AccordionContent>
                      {actions[key].map(action => (
                        <ActionComponent key={action.name} {...action} />
                      ))}
                    </AccordionContent>
                  </AccordionItem>
                ))}
              </Accordion>
            )}
          </TabsContent>
        ))}
      </div>
    </Tabs>
  );
};

export { ActionsList };
