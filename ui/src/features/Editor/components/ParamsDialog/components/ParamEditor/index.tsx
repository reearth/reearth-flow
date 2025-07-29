import { InfoIcon, NutIcon, PuzzlePieceIcon } from "@phosphor-icons/react";
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
import type { NodeData, NodeParams } from "@flow/types";

import { FieldContext } from "../../utils/fieldUtils";

type Props = {
  readonly?: boolean;
  nodeId: string;
  nodeMeta: NodeData;
  nodeType: string;
  nodeParams?: NodeParams;
  onParamsUpdate: (data: any) => void;
  onUpdate: (
    nodeId: string,
    data: any,
    type: "params" | "customizations",
  ) => Promise<void>;
  onWorkflowRename?: (id: string, name: string) => void;
  onValueEditorOpen: (fieldContext: FieldContext) => void;
};

const ParamEditor: React.FC<Props> = ({
  readonly,
  nodeId,
  nodeMeta,
  nodeParams,
  nodeType,
  onParamsUpdate,
  onUpdate,
  onWorkflowRename,
  onValueEditorOpen,
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

  const [updatedCustomization, setUpdatedCustomization] = useState(
    nodeMeta.customizations,
  );
  const [isParamsValid, setIsParamsValid] = useState(true);
  const [isCustomizationsValid, setIsCustomizationsValid] = useState(true);

  const handleCustomizationChange = (data: any) => {
    setUpdatedCustomization(data);
  };

  const handleParamsValidationChange = (isValid: boolean) => {
    setIsParamsValid(isValid);
  };

  const handleCustomizationsValidationChange = (isValid: boolean) => {
    setIsCustomizationsValid(isValid);
  };

  const [activeTab, setActiveTab] = useState(
    createdAction && !createdAction.parameter ? "customizations" : "params",
  );

  const handleUpdate = () => {
    if (activeTab === "params" && isParamsValid) {
      onUpdate(nodeId, nodeParams, "params");
    } else if (activeTab === "customizations" && isCustomizationsValid) {
      if (nodeType === "subworkflow" && nodeMeta.subworkflowId) {
        onUpdate(nodeId, updatedCustomization, "customizations");
        onWorkflowRename?.(
          nodeMeta?.subworkflowId,
          updatedCustomization?.customName || nodeMeta?.officialName,
        );
      } else {
        onUpdate(nodeId, updatedCustomization, "customizations");
      }
    }
  };

  const isCurrentTabValid =
    activeTab === "params" ? isParamsValid : isCustomizationsValid;

  return (
    <div className="flex h-[60vh] flex-col gap-4 bg-card">
      <Tabs
        onValueChange={setActiveTab}
        value={activeTab}
        className="flex h-full">
        <TabsList className="flex h-full flex-col justify-start gap-2 rounded-none bg-secondary p-2">
          {createdAction?.parameter && (
            <TabsTrigger
              className="h-[30px] w-full justify-start gap-2"
              value="params">
              <PuzzlePieceIcon className="shrink-0" />
              <p>{t("Parameters")}</p>
            </TabsTrigger>
          )}
          <TabsTrigger
            className="h-[30px] w-full justify-start gap-2"
            value="customizations">
            <NutIcon className="shrink-0" />
            <p>{t("Customizations")}</p>
          </TabsTrigger>
          <TabsTrigger
            className="h-[30px] w-full justify-start gap-2"
            value="details">
            <InfoIcon className="shrink-0" />
            <p>{t("Details")}</p>
          </TabsTrigger>
        </TabsList>
        <TabsContent className="px-6 py-4" value="params" asChild>
          <div className="flex size-full min-h-0 flex-col justify-between gap-4">
            <div className="min-h-0 overflow-scroll rounded px-2 pt-1">
              {!createdAction?.parameter && (
                <BasicBoiler
                  text={t("No Parameters Available")}
                  className="size-4 pt-16 [&>div>p]:text-sm"
                  icon={<FlowLogo className="size-12 text-accent" />}
                />
              )}
              {createdAction && (
                <SchemaForm
                  readonly={readonly}
                  schema={patchedSchemaParams}
                  defaultFormData={nodeParams}
                  onChange={onParamsUpdate}
                  onValidationChange={handleParamsValidationChange}
                  onEditorOpen={onValueEditorOpen}
                />
              )}
            </div>
            <Button
              className="shrink-0 self-end"
              size="lg"
              onClick={handleUpdate}
              disabled={readonly || !isCurrentTabValid}>
              {t("Update")}
            </Button>
          </div>
        </TabsContent>
        <TabsContent className="px-6 py-4" value="customizations" asChild>
          <div className="flex size-full min-h-0 flex-col justify-between gap-4">
            <div className="min-h-0 overflow-scroll rounded px-2 pt-4">
              {!createdAction?.customizations && (
                <BasicBoiler
                  text={t("No Customizations Available")}
                  className="size-4 pt-16 [&>div>p]:text-sm"
                  icon={<FlowLogo className="size-12 text-accent" />}
                />
              )}
              {createdAction && (
                <div className="space-y-4">
                  <h4 className="border-b text-sm font-medium">
                    {t("Customization Options")}
                  </h4>
                  <SchemaForm
                    readonly={readonly}
                    schema={createdAction?.customizations}
                    defaultFormData={updatedCustomization}
                    onChange={handleCustomizationChange}
                    onValidationChange={handleCustomizationsValidationChange}
                  />
                </div>
              )}
            </div>
            <Button
              className="shrink-0 self-end"
              size="lg"
              onClick={handleUpdate}
              disabled={readonly || !isCurrentTabValid}>
              {t("Update")}
            </Button>
          </div>
        </TabsContent>
        <TabsContent className="w-full px-6 py-4" value="details">
          <div className="min-h-32 w-full overflow-scroll rounded border px-2 pt-4">
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
                    <div className="flex items-center text-sm">
                      <p className="mr-2 w-[150px] font-medium">
                        {t("Action Name")}:
                      </p>
                      <p className="text-white">{nodeMeta.officialName}</p>
                    </div>
                    <div className="flex items-center text-sm">
                      <p className="mr-2 w-[150px] font-medium">
                        {t("Description")}:
                      </p>
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
