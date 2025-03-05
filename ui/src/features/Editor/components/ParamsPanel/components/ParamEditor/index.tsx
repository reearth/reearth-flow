import { RJSFSchema } from "@rjsf/utils";
import { JSONSchema7Definition } from "json-schema";
import { memo, useEffect, useMemo, useRef, useState } from "react";

import {
  SchemaForm,
  Button,
  Tabs,
  TabsContent,
  TabsTrigger,
  TabsList,
} from "@flow/components";
import { patchAnyOfType } from "@flow/components/SchemaForm/patchSchemaTypes";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";
import { batchNodeAction, noteNodeAction } from "@flow/lib/reactFlow";
import { generalNodeSchema } from "@flow/lib/reactFlow/nodeTypes/GeneralNode";
import type { NodeData } from "@flow/types";

type Props = {
  nodeId: string;
  nodeMeta: NodeData;
  nodeType: string;
  nodeParameters?: unknown; // TODO: define type
  onSubmit: (
    nodeId: string,
    data: any,
    type: "params" | "customization",
  ) => Promise<void>;
};

const ParamEditor: React.FC<Props> = ({
  nodeId,
  nodeMeta,
  // nodeType,
  // nodeParameters = [{ id: "param1", name: "Param 1", value: "Value 1", type: "string"}],
  onSubmit,
}) => {
  const t = useT();
  const { useGetActionById } = useAction(i18n.language);
  let { action } = useGetActionById(nodeMeta.officialName);
  const firstRenderRef = useRef(true);

  // For nodes such as note and batch that are not in the actions list and therefore have no params
  if (!action) {
    switch (nodeMeta.officialName) {
      case "batch":
        action = {
          ...nodeMeta,
          ...batchNodeAction,
        };
        break;

      case "note":
        action = {
          ...nodeMeta,
          ...noteNodeAction,
        };
        break;

      default:
        action = undefined;
    }
  }

  // Only set the default customization schema if it doesn't exist yet
  const [actionWithCustomization, setActionWithCustomization] =
    useState(action);

  useEffect(() => {
    if (firstRenderRef.current && action) {
      firstRenderRef.current = false;

      // Only update if we need to add customization
      if (!action.customization) {
        setActionWithCustomization({
          ...action,
          customization: generalNodeSchema,
        });
      } else {
        setActionWithCustomization(action);
      }
    }
  }, [action]);

  // This is a patch for the `anyOf` type in JSON Schema.
  const patchedSchemaParams = useMemo<RJSFSchema | undefined>(
    () =>
      actionWithCustomization?.parameter
        ? patchAnyOfType(
            actionWithCustomization.parameter as JSONSchema7Definition,
          )
        : undefined,
    [actionWithCustomization?.parameter],
  );

  const patchedSchemaCustomization = useMemo<RJSFSchema | undefined>(
    () =>
      actionWithCustomization?.customization
        ? patchAnyOfType(
            actionWithCustomization.customization as JSONSchema7Definition,
          )
        : undefined,
    [actionWithCustomization?.customization],
  );

  const [updatedParams, setUpdatedParams] = useState(nodeMeta.params);
  const [updatedCustomization, setUpdatedCustomization] = useState(
    nodeMeta.customization,
  );

  const handleParamChange = (data: any) => {
    setUpdatedParams(data);
  };

  const handleCustomizationChange = (data: any) => {
    setUpdatedCustomization(data);
  };

  const [activeTab, setActiveTab] = useState("params");

  const handleSubmit = () => {
    if (activeTab === "params") {
      onSubmit(nodeId, updatedParams, "params");
    } else {
      onSubmit(nodeId, updatedCustomization, "customization");
    }
  };

  return (
    <div className="flex h-full flex-col gap-4">
      <Tabs
        defaultValue="params"
        onValueChange={setActiveTab}
        value={activeTab}
        className="flex h-full flex-col gap-4">
        <div className="flex justify-between gap-2">
          <p className="text-lg dark:font-thin">
            {activeTab === "params" ? t("Parameters") : t("Node Customization")}
          </p>
          <Button onClick={handleSubmit}>{t("Submit")}</Button>
        </div>
        <TabsList className="flex justify-between">
          <TabsTrigger className="flex-1" value="params">
            {t("Parameters")}
          </TabsTrigger>
          <TabsTrigger className="flex-1" value="customization">
            {t("Node Customization")}
          </TabsTrigger>
        </TabsList>
        <TabsContent value="params">
          <div className="min-h-0 overflow-scroll rounded border bg-card px-2">
            {!actionWithCustomization?.parameter && (
              <p>{t("No Parameters Available")}</p>
            )}
            {actionWithCustomization && (
              <SchemaForm
                schema={patchedSchemaParams}
                defaultFormData={updatedParams}
                onChange={handleParamChange}
              />
            )}
          </div>
        </TabsContent>
        <TabsContent value="customization">
          <div className="min-h-0 overflow-scroll rounded border bg-card px-2">
            {!actionWithCustomization?.customization && (
              <p>{t("No Customization Available")}</p>
            )}
            {actionWithCustomization && (
              <div>
                <div className="text-sm leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                  <p>{t("Node Details")}</p>
                  <p>
                    {t("Action Name")}: {action?.name}
                  </p>
                </div>
                <SchemaForm
                  schema={patchedSchemaCustomization}
                  defaultFormData={updatedCustomization}
                  onChange={handleCustomizationChange}
                />
              </div>
            )}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default memo(ParamEditor);
