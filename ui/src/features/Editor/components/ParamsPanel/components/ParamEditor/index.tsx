import { RJSFSchema } from "@rjsf/utils";
import { JSONSchema7Definition } from "json-schema";
import { memo, useMemo, useState } from "react";

import {
  SchemaForm,
  Button,
  Tabs,
  TabsContent,
  TabsTrigger,
  TabsList,
  FlowLogo,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { patchAnyOfType } from "@flow/components/SchemaForm/patchSchemaTypes";
import { useNodeSchemaGenerate } from "@flow/hooks";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";
import type { NodeData } from "@flow/types";

type Props = {
  nodeId: string;
  nodeMeta: NodeData;
  nodeType: string;
  nodeParameters?: unknown; // TODO: define type
  onSubmit: (
    nodeId: string,
    data: any,
    type: "params" | "customizations",
  ) => Promise<void>;
};

const ParamEditor: React.FC<Props> = ({
  nodeId,
  nodeMeta,
  nodeType,
  // nodeParameters = [{ id: "param1", name: "Param 1", value: "Value 1", type: "string"}],
  onSubmit,
}) => {
  const t = useT();
  const { useGetActionById } = useAction(i18n.language);
  const { action: fetchedAction } = useGetActionById(nodeMeta.officialName);

  // Used to generate the customization schema for the node with translations
  const { action: createdAction } = useNodeSchemaGenerate(
    nodeType,
    nodeMeta,
    fetchedAction,
  );

  // This is a patch for the `anyOf` type in JSON Schema.
  const patchedSchemaParams = useMemo<RJSFSchema | undefined>(
    () =>
      createdAction?.parameter
        ? patchAnyOfType(createdAction.parameter as JSONSchema7Definition)
        : undefined,
    [createdAction?.parameter],
  );

  const [updatedParams, setUpdatedParams] = useState(nodeMeta.params);
  const [updatedCustomization, setUpdatedCustomization] = useState(
    nodeMeta.customizations,
  );

  const handleParamChange = (data: any) => {
    setUpdatedParams(data);
  };

  const handleCustomizationChange = (data: any) => {
    setUpdatedCustomization(data);
  };

  const [activeTab, setActiveTab] = useState(
    createdAction?.name === "batch" || createdAction?.name === "note"
      ? "customization"
      : "params",
  );

  const handleSubmit = () => {
    if (activeTab === "params") {
      onSubmit(nodeId, updatedParams, "params");
    } else {
      onSubmit(nodeId, updatedCustomization, "customizations");
    }
  };

  return (
    <div className="flex h-full flex-col gap-4">
      <Tabs
        defaultValue={
          createdAction?.name === "batch" || createdAction?.name === "note"
            ? "customization"
            : "params"
        }
        onValueChange={setActiveTab}
        value={activeTab}
        className="flex h-full flex-col gap-4">
        <div className="flex justify-between gap-2">
          <p className="text-lg dark:font-thin">
            {activeTab === "params"
              ? t("Parameters")
              : activeTab === "customization"
                ? t("Customization")
                : t("Details")}
          </p>
          {activeTab !== "details" && (
            <Button onClick={handleSubmit}>{t("Submit")}</Button>
          )}
          {activeTab === "details" && <div className="h-[36px]" />}
        </div>
        <TabsList className="flex justify-between gap-2">
          {createdAction?.name !== "batch" &&
            createdAction?.name !== "note" && (
              <TabsTrigger className="flex-1" value="params">
                {t("Parameters")}
              </TabsTrigger>
            )}
          <TabsTrigger className="flex-1" value="customization">
            {t("Customization")}
          </TabsTrigger>
          <TabsTrigger className="flex-1" value="details">
            {t("Details")}
          </TabsTrigger>
        </TabsList>
        <TabsContent value="params">
          <div className="min-h-32 overflow-scroll rounded border bg-card px-2 pt-1">
            {!createdAction?.parameter && (
              <BasicBoiler
                text={t("No Parameters Available")}
                className="size-4 pt-16 [&>div>p]:text-sm"
                icon={<FlowLogo className="size-12 text-accent" />}
              />
            )}
            {createdAction && (
              <SchemaForm
                schema={patchedSchemaParams}
                defaultFormData={updatedParams}
                onChange={handleParamChange}
              />
            )}
          </div>
        </TabsContent>
        <TabsContent value="customization">
          <div className="min-h-32 overflow-scroll rounded border bg-card px-2 pt-4">
            {!createdAction?.customizations && (
              <BasicBoiler
                text={t("No Customization Available")}
                className="size-4 pt-16 [&>div>p]:text-sm"
                icon={<FlowLogo className="size-12 text-accent" />}
              />
            )}
            {createdAction && (
              <div className="space-y-4">
                <div>
                  <h4 className="border-b text-sm font-medium">
                    {t("Customization Options")}
                  </h4>
                  <SchemaForm
                    schema={createdAction?.customizations}
                    defaultFormData={updatedCustomization}
                    onChange={handleCustomizationChange}
                  />
                </div>
              </div>
            )}
          </div>
        </TabsContent>
        <TabsContent value="details">
          <div className="min-h-32 overflow-scroll rounded border bg-card px-2 pt-4">
            {!createdAction && (
              <BasicBoiler
                text={t("No Details Available")}
                className="size-4 pt-16 [&>div>p]:text-sm"
                icon={<FlowLogo className="size-12 text-accent" />}
              />
            )}
            {createdAction && (
              <div className="space-y-4">
                <div className="rounded-md ">
                  <h4 className="border-b text-sm font-medium">
                    {t("Node Details")}
                  </h4>
                  <div className="my-4 flex w-full flex-col gap-4">
                    <p className="flex items-center text-sm">
                      <span className="mr-2 font-medium">
                        {t("Action Name")}:
                      </span>
                      <span className="text-white">
                        {nodeMeta.officialName}
                      </span>
                    </p>
                    <div className="flex flex-col gap-2">
                      <span className="mr-2 text-sm font-medium">
                        {t("Description")}:
                      </span>
                      {createdAction?.description && (
                        <p className="text-sm">{createdAction.description}</p>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default memo(ParamEditor);
