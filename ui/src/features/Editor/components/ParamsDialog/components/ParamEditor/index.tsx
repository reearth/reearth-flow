import {
  InfoIcon,
  NutIcon,
  PuzzlePieceIcon,
  QuestionIcon,
} from "@phosphor-icons/react";
import { RJSFSchema } from "@rjsf/utils";
import { memo, useEffect, useState } from "react";

import {
  SchemaForm,
  Button,
  Tabs,
  TabsContent,
  TabsTrigger,
  TabsList,
  FlowLogo,
  Tooltip,
  TooltipTrigger,
  TooltipContent,
  ActionDetails,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useNodeSchemaGenerate } from "@flow/hooks";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";
import type { AwarenessUser, NodeData, NodeParams } from "@flow/types";

import { extractDescriptions } from "../../utils/extractDescriptions";
import { FieldContext } from "../../utils/fieldUtils";
import { schemasMatch } from "../../utils/schemaFingerprint";

import SchemaMigrationView from "./SchemaMigrationView";

type Props = {
  readonly?: boolean;
  nodeId: string;
  nodeMeta: NodeData;
  nodeType: string;
  nodeParams?: NodeParams;
  nodeCustomizations?: any;
  fieldFocusMap?: Record<string, AwarenessUser[]>;
  onParamsUpdate: (data: any, changedFieldId?: string) => void;
  onCustomizationsUpdate: (data: any, changedFieldId?: string) => void;
  onUpdate: (
    nodeId: string,
    updatedParams: any,
    updatedCustomizations: any,
    paramsSchema?: RJSFSchema,
  ) => Promise<void>;
  onMigrate: (
    nodeId: string,
    newParams: NodeParams,
    paramsSchema?: RJSFSchema,
  ) => void;
  onWorkflowRename?: (id: string, name: string) => void;
  onParamFieldFocus?: (fieldId: string | null) => void;
  onValueEditorOpen: (fieldContext: FieldContext) => void;
  onPythonEditorOpen?: (fieldContext: FieldContext) => void;
  onFlowExprEditorOpen?: (fieldContext: FieldContext) => void;
};

const ParamEditor: React.FC<Props> = ({
  readonly,
  nodeId,
  nodeMeta,
  nodeParams,
  nodeCustomizations,
  nodeType,
  fieldFocusMap,
  onParamsUpdate,
  onCustomizationsUpdate,
  onUpdate,
  onMigrate,
  onWorkflowRename,
  onParamFieldFocus,
  onValueEditorOpen,
  onPythonEditorOpen,
  onFlowExprEditorOpen,
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

  // Generate UI schema from original schema (before patching) to preserve Expr detection
  const originalSchema = createdAction?.parameter;

  const needsMigration =
    !!createdAction?.parameter &&
    !schemasMatch(nodeMeta.paramsSchema, createdAction.parameter);

  const [migrationComplete, setMigrationComplete] = useState(false);

  useEffect(() => {
    setMigrationComplete(false);
  }, [nodeId]);

  const [isParamsValid, setIsParamsValid] = useState(true);
  const [isCustomizationsValid, setIsCustomizationsValid] = useState(true);

  const handleParamsValidationChange = (isValid: boolean) => {
    setIsParamsValid(isValid);
  };

  const handleCustomizationsValidationChange = (isValid: boolean) => {
    setIsCustomizationsValid(isValid);
  };

  const [activeTab, setActiveTab] = useState(
    createdAction && !createdAction.parameter ? "customizations" : "params",
  );

  useEffect(() => {
    if (activeTab === "params" && createdAction && !createdAction.parameter) {
      setActiveTab("customizations");
    }
  }, [activeTab, createdAction]);

  const isCurrentTabValid =
    activeTab === "params" ? isParamsValid : isCustomizationsValid;

  const handleUpdate = () => {
    if (!isCurrentTabValid) return;
    if (nodeType === "subworkflow" && nodeMeta.subworkflowId) {
      onWorkflowRename?.(
        nodeMeta?.subworkflowId,
        nodeCustomizations?.customName || nodeMeta?.officialName,
      );
    }
    onUpdate(nodeId, nodeParams, nodeCustomizations, createdAction?.parameter);
  };

  const handleFormKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (readonly || e.key !== "Enter") return;
    if (
      e.nativeEvent.isComposing ||
      e.shiftKey ||
      e.metaKey ||
      e.ctrlKey ||
      e.altKey
    )
      return;
    if ((e.target as HTMLElement).tagName !== "INPUT") return;
    e.preventDefault();
    handleUpdate();
  };

  const handleMigrate = (newParams: NodeParams) => {
    setMigrationComplete(true);
    onMigrate(nodeId, newParams, createdAction?.parameter);
  };

  const customizationDescriptions = extractDescriptions(
    createdAction?.customizations,
  );

  return (
    <div className="relative flex h-[60vh] flex-col gap-4">
      <Tabs
        onValueChange={setActiveTab}
        value={activeTab}
        className="flex h-full">
        <TabsList className="mx-2 mb-2 flex h-[calc(100%-0.5rem)] flex-col justify-start gap-2 rounded-xl border border-primary/50 bg-secondary p-2 shadow-md shadow-secondary backdrop-blur-xs">
          {createdAction?.parameter && (
            <TabsTrigger
              className="h-[30px] w-full justify-start gap-2 rounded-xl px-2 py-0.5"
              value="params">
              <PuzzlePieceIcon className="shrink-0" />
              <p>{t("Parameters")}</p>
            </TabsTrigger>
          )}
          <TabsTrigger
            className="h-[30px] w-full justify-start gap-2 rounded-xl px-2 py-0.5"
            value="customizations">
            <NutIcon className="shrink-0" />
            <p>{t("Customizations")}</p>
          </TabsTrigger>
          <TabsTrigger
            className="h-[30px] w-full justify-start gap-2 rounded-xl px-2 py-0.5"
            value="details">
            <InfoIcon className="shrink-0" />
            <p>{t("Details")}</p>
          </TabsTrigger>
        </TabsList>
        <TabsContent
          className="px-4 pb-2"
          value="params"
          render={
            <div
              className="flex size-full min-h-0 min-w-0 flex-col justify-between gap-4"
              onKeyDown={handleFormKeyDown}
            />
          }>
          <div className="min-h-0 min-w-0 overflow-scroll rounded px-2">
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
                schema={originalSchema}
                actionName={nodeMeta.officialName}
                defaultFormData={nodeParams}
                fieldFocusMap={fieldFocusMap}
                onFieldFocus={onParamFieldFocus}
                onChange={onParamsUpdate}
                onValidationChange={handleParamsValidationChange}
                onEditorOpen={onValueEditorOpen}
                onPythonEditorOpen={onPythonEditorOpen}
                onFlowExprEditorOpen={onFlowExprEditorOpen}
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
        </TabsContent>
        <TabsContent
          className="px-4 pb-2"
          value="customizations"
          render={
            <div
              className="flex size-full min-h-0 min-w-0 flex-col justify-between gap-4"
              onKeyDown={handleFormKeyDown}
            />
          }>
          <div className="min-h-0 min-w-0 overflow-scroll rounded px-2">
            {!createdAction?.customizations && (
              <BasicBoiler
                text={t("No Customizations Available")}
                className="size-4 pt-16 [&>div>p]:text-sm"
                icon={<FlowLogo className="size-12 text-accent" />}
              />
            )}
            {createdAction && (
              <div>
                <div className="mb-1 flex items-center gap-1">
                  <p className="text-sm font-bold">
                    {t("Customization Options")}
                  </p>
                  <Tooltip>
                    <TooltipTrigger
                      render={
                        <div className="cursor-pointer p-1">
                          <QuestionIcon className="h-5 w-5" weight="thin" />
                        </div>
                      }
                    />
                    <TooltipContent
                      side="top"
                      align="start"
                      className="bg-primary">
                      <div className="max-w-75 text-xs text-muted-foreground">
                        {Object.entries(customizationDescriptions).map(
                          ([key, value], index) => (
                            <div key={index}>
                              <span className="font-medium">{key}:</span>{" "}
                              {String(value)}
                            </div>
                          ),
                        )}
                      </div>
                    </TooltipContent>
                  </Tooltip>
                </div>
                <div className="border-b" />
                <SchemaForm
                  readonly={readonly}
                  schema={createdAction?.customizations}
                  defaultFormData={nodeCustomizations}
                  fieldFocusMap={fieldFocusMap}
                  onFieldFocus={onParamFieldFocus}
                  onChange={onCustomizationsUpdate}
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
        </TabsContent>
        <TabsContent className="w-full px-4" value="details">
          <div className="min-h-32 w-full overflow-scroll rounded">
            {!createdAction && (
              <BasicBoiler
                text={t("No Details Available")}
                className="size-4 pt-16 [&>div>p]:text-sm"
                icon={<FlowLogo className="size-12 text-accent" />}
              />
            )}
            {createdAction && (
              <div className="pr-6 pl-4">
                <ActionDetails action={createdAction} />
              </div>
            )}
          </div>
        </TabsContent>
      </Tabs>

      {needsMigration && !migrationComplete && (
        <div className="absolute inset-0 z-10 rounded bg-background">
          <SchemaMigrationView
            readonly={readonly}
            storedSchema={nodeMeta.paramsSchema}
            storedParams={nodeMeta.params}
            newSchema={originalSchema}
            actionName={nodeMeta.officialName}
            fieldFocusMap={fieldFocusMap}
            onParamFieldFocus={onParamFieldFocus}
            onMigrate={handleMigrate}
            onValueEditorOpen={onValueEditorOpen}
            onPythonEditorOpen={onPythonEditorOpen}
            onFlowExprEditorOpen={onFlowExprEditorOpen}
          />
        </div>
      )}
    </div>
  );
};

export default memo(ParamEditor);
