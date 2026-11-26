import {
  PencilLineIcon,
  CornersInIcon,
  CornersOutIcon,
  FileIcon,
  CircleIcon,
} from "@phosphor-icons/react";
import { useCallback, useId, useMemo, useRef, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  Label,
  Switch,
  TextArea,
  Tabs,
  TabsContent,
} from "@flow/components";
import type { EditorContext as FieldContext } from "@flow/components/SchemaForm";
import AssetPickerDialog from "@flow/features/AssetPickerDialog";
import { useWorkflowVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";

import FlowExprCodeEditor from "../ValueEditorDialog/components/FlowExprCodeEditor";
import type { FlowExprCodeEditorRef } from "../ValueEditorDialog/components/FlowExprCodeEditor";
import type { AutocompleteSuggestion } from "../ValueEditorDialog/components/flowExprConstants";
import { toVariableAutocompleteSuggestions } from "../ValueEditorDialog/components/variableAutocomplete";

export type CodeValue = {
  type: "flowExpr" | "string";
  value: string;
};

type Props = {
  open: boolean;
  fieldContext: FieldContext;
  attributeSuggestions?: AutocompleteSuggestion[];
  onClose: () => void;
  onValueSubmit?: (value: CodeValue) => void;
};

const FlowExprEditorDialog: React.FC<Props> = ({
  open,
  fieldContext,
  attributeSuggestions,
  onClose,
  onValueSubmit,
}) => {
  const t = useT();

  const initialCode = fieldContext.value as CodeValue | undefined;

  const allowedTypes = (fieldContext.schema as any)?.properties?.type?.enum as
    string[] | undefined;
  const flowExprAllowed = !allowedTypes || allowedTypes.includes("flowExpr");
  const stringAllowed = !allowedTypes || allowedTypes.includes("string");

  const defaultType: "flowExpr" | "string" = flowExprAllowed
    ? "flowExpr"
    : "string";
  const [codeType, setCodeType] = useState<"flowExpr" | "string">(
    initialCode?.type === "flowExpr" && flowExprAllowed
      ? "flowExpr"
      : initialCode?.type === "string" && stringAllowed
        ? "string"
        : defaultType,
  );
  const [codeValue, setCodeValue] = useState(initialCode?.value ?? "");
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [showAssetPicker, setShowAssetPicker] = useState(false);

  const editorRef = useRef<FlowExprCodeEditorRef>(null);
  const expressionSwitchId = useId();

  const [currentProject] = useCurrentProject();
  const { useGetWorkflowVariables } = useWorkflowVariables();
  const { workflowVariables } = useGetWorkflowVariables(currentProject?.id);
  const variableSuggestions = useMemo(
    () => toVariableAutocompleteSuggestions(workflowVariables),
    [workflowVariables],
  );

  const insertAtCursor = useCallback(
    (text: string) => {
      if (codeType === "flowExpr" && editorRef.current) {
        editorRef.current.insertAtCursor(text);
      } else {
        setCodeValue((prev) => prev + text);
      }
    },
    [codeType],
  );

  const insertUrl = useCallback(
    (url: string) => {
      insertAtCursor(codeType === "flowExpr" ? JSON.stringify(url) : url);
      setShowAssetPicker(false);
    },
    [codeType, insertAtCursor],
  );

  const handleVariableSelect = useCallback(
    (variableName: string) => {
      insertAtCursor(`variables["${variableName}"]`);
    },
    [insertAtCursor],
  );

  const handleSubmit = useCallback(() => {
    if (!onValueSubmit) return;
    onValueSubmit({ type: codeType, value: codeValue });
    onClose();
  }, [codeType, codeValue, onValueSubmit, onClose]);

  const handleFullscreenToggle = useCallback(() => {
    setIsFullscreen((prev) => !prev);
  }, []);

  const fieldLabel =
    fieldContext.schema?.title || fieldContext.fieldName || t("Unknown Field");

  return (
    <>
      <Dialog open={open} disablePointerDismissal onOpenChange={onClose}>
        <DialogContent size={isFullscreen ? "full" : "3xl"} hideCloseButton>
          <DialogHeader>
            <DialogTitle className="relative flex h-[52px] items-center justify-between">
              <div className="flex flex-1 gap-4">
                <div className="flex items-center gap-2">
                  <PencilLineIcon weight="thin" />
                  {t("FlowExpr Editor")} — {fieldLabel}
                </div>
                <div className="flex flex-1 items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowAssetPicker(true)}>
                    <FileIcon className="h-4 w-4" />
                    {t("Assets")}
                  </Button>
                  {workflowVariables &&
                    workflowVariables.length > 0 &&
                    flowExprAllowed &&
                    codeType === "flowExpr" && (
                      <DropdownMenu>
                        <DropdownMenuTrigger
                          render={
                            <Button variant="outline" size="sm">
                              <CircleIcon className="h-4 w-4" />
                              {t("Variables")}
                            </Button>
                          }
                        />
                        <DropdownMenuContent align="end" className="w-64">
                          {workflowVariables.map((variable) => (
                            <DropdownMenuItem
                              key={variable.id}
                              onClick={() =>
                                handleVariableSelect(variable.name)
                              }
                              className="flex flex-col items-start">
                              <div className="font-mono text-sm">
                                {variable.name}
                              </div>
                              <div className="text-xs text-muted-foreground">
                                {variable.type} •{" "}
                                {variable.defaultValue || t("No value set")}
                              </div>
                            </DropdownMenuItem>
                          ))}
                        </DropdownMenuContent>
                      </DropdownMenu>
                    )}
                </div>
              </div>
              <IconButton
                className="-mr-2 rounded-[4px]"
                tooltipText={
                  isFullscreen ? t("Exit fullscreen") : t("Enter fullscreen")
                }
                tooltipOffset={6}
                tooltipPosition="left"
                icon={
                  isFullscreen ? (
                    <CornersInIcon weight="thin" size={18} />
                  ) : (
                    <CornersOutIcon weight="thin" size={18} />
                  )
                }
                onClick={handleFullscreenToggle}
              />
            </DialogTitle>
          </DialogHeader>

          <Tabs
            value={codeType}
            onValueChange={(v) => setCodeType(v as "flowExpr" | "string")}
            className={`flex flex-col ${isFullscreen ? "h-[calc(100vh-52px)]" : "h-[70vh]"}`}>
            {flowExprAllowed && stringAllowed && (
              <div className="flex shrink-0 items-center gap-2 border-b px-4 py-2">
                <Switch
                  id={expressionSwitchId}
                  checked={codeType === "flowExpr"}
                  onCheckedChange={(checked) =>
                    setCodeType(checked ? "flowExpr" : "string")
                  }
                />
                <Label htmlFor={expressionSwitchId}>{t("Expression")}</Label>
              </div>
            )}

            {/* Editor areas */}
            <TabsContent
              value="flowExpr"
              className="relative mt-0 min-h-0 flex-1 overflow-hidden border-b">
              <FlowExprCodeEditor
                ref={editorRef}
                className="size-full"
                value={codeValue}
                onChange={setCodeValue}
                attributeSuggestions={attributeSuggestions}
                variableSuggestions={variableSuggestions}
                placeholder={t('e.g. variables["BASE_DIR"] + "/filename"')}
              />
            </TabsContent>
            <TabsContent
              value="string"
              className="mt-0 min-h-0 flex-1 border-b">
              <TextArea
                className="size-full resize-none rounded-none border-0 focus-visible:ring-0"
                value={codeValue}
                onChange={(e) => setCodeValue(e.target.value)}
                placeholder={t("Enter a literal string value")}
                spellCheck={false}
              />
            </TabsContent>

            {/* Hint bar */}
            <div className="shrink-0 bg-muted/20 px-4 py-2 text-xs text-muted-foreground">
              {codeType === "flowExpr" ? (
                <span>
                  {t(
                    'FlowExpr: variables["VAR"], attributes["attr"], "a" + "b", math.sin(...). API is still stabilising.',
                  )}
                </span>
              ) : (
                <span>{t("Literal string: no expression evaluation")}</span>
              )}
            </div>

            <DialogFooter className="p-4">
              <div className="flex justify-end gap-2">
                <Button variant="outline" onClick={onClose}>
                  {t("Cancel")}
                </Button>
                <Button onClick={handleSubmit}>{t("Apply")}</Button>
              </div>
            </DialogFooter>
          </Tabs>
        </DialogContent>
      </Dialog>

      {showAssetPicker && (
        <AssetPickerDialog
          onDialogClose={() => setShowAssetPicker(false)}
          onSelect={insertUrl}
        />
      )}
    </>
  );
};

export default FlowExprEditorDialog;
