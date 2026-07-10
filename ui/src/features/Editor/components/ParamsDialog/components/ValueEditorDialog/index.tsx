import {
  PencilLineIcon,
  CircleIcon,
  CornersInIcon,
  CornersOutIcon,
  FileIcon,
} from "@phosphor-icons/react";
import { QuestionIcon } from "@phosphor-icons/react/dist/ssr";
import { useCallback, useState, useRef } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DialogFooter,
  IconButton,
  CmsLogo,
} from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import CmsIntegrationDialog from "@flow/features/CmsIntegrationDialog";
import { useWorkflowVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import { Asset } from "@flow/types";

import { FieldContext } from "../../utils/fieldUtils";

import FlowExprCodeEditor, {
  type FlowExprCodeEditorRef,
} from "./components/FlowExprCodeEditor";

type Props = {
  open: boolean;
  fieldContext: FieldContext;
  onClose: () => void;
  onValueSubmit?: (value: any) => void;
};

export type DialogOptions = "assets" | "cms" | undefined;

const ValueEditorDialog: React.FC<Props> = ({
  open,
  fieldContext,
  onClose,
  onValueSubmit,
}) => {
  const t = useT();
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);
  const handleDialogOpen = (dialog: DialogOptions) => setShowDialog(dialog);
  const handleDialogClose = () => setShowDialog(undefined);
  const [value, setValue] = useState(fieldContext.value);

  // Fullscreen state
  const [isFullscreen, setIsFullscreen] = useState(false);

  // Ref for FlowExprCodeEditor to enable cursor insertion
  const editorRef = useRef<FlowExprCodeEditorRef>(null);

  const [currentProject] = useCurrentProject();

  const { useGetWorkflowVariables } = useWorkflowVariables();

  const { workflowVariables } = useGetWorkflowVariables(currentProject?.id);

  const handleSubmit = useCallback(() => {
    if (!onValueSubmit) return;

    try {
      // Try to parse as JSON first for complex values
      let parsedValue: any = value;
      if (
        fieldContext?.schema?.type === "object" ||
        fieldContext?.schema?.type === "array"
      ) {
        parsedValue = JSON.parse(value);
      } else if (fieldContext?.schema?.type === "number") {
        parsedValue = Number(value);
      } else if (fieldContext?.schema?.type === "integer") {
        parsedValue = parseInt(value, 10);
      } else if (fieldContext?.schema?.type === "boolean") {
        parsedValue = value === "true";
      }

      onValueSubmit(parsedValue);
      onClose();
    } catch (_error) {
      // If JSON parsing fails, use the raw string value
      onValueSubmit(value);
      onClose();
    }
  }, [value, onValueSubmit, onClose, fieldContext?.schema?.type]);

  const getFieldTypeDisplay = (schema: any) => {
    if (schema?.type) {
      return schema.type;
    }
    if (schema?.format) {
      return schema.format;
    }
    return "text";
  };

  const fieldType = getFieldTypeDisplay(fieldContext.schema);

  const handleWorkflowVariableSet = useCallback((variable: any) => {
    const v = `env.get("${variable.name}")`;
    setValue(v);
  }, []);

  const handleAssetDoubleClick = (asset: Asset) => {
    const v = asset.url;
    setValue?.(v);
  };

  const handleCmsItemValue = (cmsItemAssetUrl: string) => {
    setValue?.(cmsItemAssetUrl);
    handleDialogClose();
  };

  const handleFullscreenToggle = useCallback(() => {
    setIsFullscreen((prev) => !prev);
  }, []);

  return (
    <>
      <Dialog open={open} disablePointerDismissal onOpenChange={onClose}>
        <DialogContent size={isFullscreen ? "full" : "3xl"} hideCloseButton>
          <DialogHeader>
            <DialogTitle className="relative flex h-[52px] items-center justify-between">
              <div className="flex flex-1 gap-4">
                <div className="flex items-center gap-2">
                  <PencilLineIcon weight="thin" />
                  {t("Value Editor")} -{" "}
                  {fieldContext.schema.title ||
                    fieldContext?.fieldName ||
                    t("Unknown Field")}{" "}
                  {fieldType ? `(${fieldType})` : ""}
                </div>
                <div className="flex flex-1 items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleDialogOpen("assets")}>
                    <FileIcon className="h-4 w-4" />
                    {t("Workspace Assets")}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleDialogOpen("cms")}>
                    <CmsLogo className="h-4 w-4" />
                    {t("CMS Integration")}
                  </Button>
                  {workflowVariables && workflowVariables.length > 0 && (
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
                            onClick={() => handleWorkflowVariableSet(variable)}
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
              <div className="-mr-2 flex items-center">
                <IconButton
                  className="rounded-[4px]"
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
              </div>
            </DialogTitle>
          </DialogHeader>

          <div
            className={`flex flex-col ${isFullscreen ? "h-[calc(100vh-52px)]" : "h-[600px]"}`}>
            <div className="relative flex-1 border-b">
              <FlowExprCodeEditor
                ref={editorRef}
                className="h-full rounded-none bg-card/20 backdrop-blur-sm"
                placeholder={t("Enter expression...")}
                value={value}
                onChange={setValue}
                data-testid="value-editor-textarea"
                aria-label={t("Expression Editor")}
                data-placeholder={t("Enter expression...")}
              />
              <Tooltip>
                <TooltipTrigger
                  render={
                    <div className="absolute right-2 bottom-2 cursor-pointer p-1">
                      <QuestionIcon className="h-6 w-6" weight="thin" />
                    </div>
                  }
                />
                <TooltipContent side="top" align="end">
                  <p className="text-sm">{t("Expression Editor Help")}</p>
                  <p className="mt-1 max-w-[200px] text-xs text-muted-foreground">
                    {t("Write FlowExpr expressions directly.")}
                  </p>
                </TooltipContent>
              </Tooltip>
            </div>
            <DialogFooter className="flex justify-end gap-2 p-4">
              <Button variant="outline" onClick={onClose}>
                {t("Cancel")}
              </Button>
              <Button onClick={handleSubmit}>{t("Submit")}</Button>
            </DialogFooter>
          </div>
        </DialogContent>
      </Dialog>
      {showDialog === "assets" && fieldContext && (
        <AssetsDialog
          onDialogClose={handleDialogClose}
          onAssetSelect={handleAssetDoubleClick}
        />
      )}
      {showDialog === "cms" && fieldContext && (
        <CmsIntegrationDialog
          onDialogClose={handleDialogClose}
          onCmsItemValue={handleCmsItemValue}
        />
      )}
    </>
  );
};

export default ValueEditorDialog;
