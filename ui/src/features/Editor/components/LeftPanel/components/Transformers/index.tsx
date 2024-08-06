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
import { useTransformer } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import { Transformer, TransformersSegregated, Segregated } from "@flow/types";

import { TransformerComponent } from "./Transformer";

type Ordering = "default" | "categorically" | "byType";

const TransformersList: React.FC = () => {
  const t = useT();
  const { useGetTransformers, useGetTransformerSegregated } = useTransformer();

  const [transformers, setTransformers] = useState<Transformer[] | undefined>();

  const [transformersSegregated, setTransformersSegregated] = useState<
    Segregated | undefined
  >();

  const { transformers: transformersData } = useGetTransformers();
  const { transformers: transformersSegregatedData } =
    useGetTransformerSegregated();

  useEffect(() => {
    if (transformersData) setTransformers(transformersData);
    if (transformersSegregatedData)
      setTransformersSegregated(transformersSegregatedData);
  }, [transformersData, transformersSegregatedData]);

  const tabs: {
    title: string;
    order: Ordering;
    transformers: Transformer[] | TransformersSegregated | undefined;
  }[] = [
    {
      title: t("All"),
      order: "default",
      transformers,
    },
    {
      title: t("Category"),
      order: "categorically",
      transformers: transformersSegregated?.byCategory,
    },
    {
      title: t("Type"),
      order: "byType",
      transformers: transformersSegregated?.byType,
    },
  ];

  const getFilteredTransformers = useCallback(
    (filter: string, transformer?: Transformer[]): Transformer[] | undefined =>
      transformer?.filter((transformer) =>
        Object.values(transformer)
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
      setTransformers(transformersData);
      setTransformersSegregated(transformersSegregatedData);
      return;
    }

    const filteredTransformers =
      transformersData && getFilteredTransformers(filter, transformersData);
    setTransformers(filteredTransformers);

    const filteredtransformersSegregated =
      transformersSegregatedData &&
      Object.keys(transformersSegregatedData).reduce((obj, rootKey) => {
        obj[rootKey] = Object.keys(transformersSegregatedData[rootKey]).reduce(
          (obj: Record<string, Transformer[] | undefined>, key) => {
            obj[key] = getFilteredTransformers(
              filter,
              transformersSegregatedData[rootKey][key]
            );
            return obj;
          },
          {}
        );
        return obj;
      }, {} as Segregated);

    setTransformersSegregated(filteredtransformersSegregated);
  }, 200);

  return (
    <Tabs defaultValue={tabs[0].order}>
      <div className="absolute w-full bg-secondary p-2">
        <TabsList className="flex justify-between px-0">
          {tabs.map(({ title, order }) => (
            <TabsTrigger
              key={order}
              value={order}
              className="w-[31%] uppercase"
            >
              {title}
            </TabsTrigger>
          ))}
        </TabsList>
        <div>
          <Input
            className="mx-auto mt-2 w-full px-2"
            placeholder={t("Search")}
            // value={search}
            onChange={(e) => handleSearch(e.target.value)}
          />
        </div>
      </div>
      <div className="mt-20 p-2">
        {tabs.map(({ order, transformers }) => (
          <TabsContent className="dark" key={order} value={order}>
            {Array.isArray(transformers) ? (
              transformers.map((transformer) => (
                <TransformerComponent key={transformer.name} {...transformer} />
              ))
            ) : (
              <Accordion type="single" collapsible>
                {transformers ? (
                  Object.keys(transformers).map((key) => (
                    <AccordionItem key={key} value={key}>
                      <AccordionTrigger>{key}</AccordionTrigger>
                      <AccordionContent>
                        {transformers[key]?.map((transformer) => (
                          <TransformerComponent
                            key={transformer.name}
                            {...transformer}
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

export { TransformersList };
