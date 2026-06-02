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

import RhaiCodeEditor, {
  type RhaiCodeEditorRef,
} from "./components/RhaiCodeEditor";
import {
  TemplateLibraryDialog,
  TemplatePlaceholderDialog,
  type ExpressionTemplate,
} from "./templates";

type Props = {
  open: boolean;
  fieldContext: FieldContext;
  onClose: () => void;
  onValueSubmit?: (value: any) => void;
};

export type DialogOptions = "assets" | "cms" | "templates" | undefined;

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

  // Template-related state
  const [selectedTemplate, setSelectedTemplate] =
    useState<ExpressionTemplate | null>(null);
  const [showPlaceholderDialog, setShowPlaceholderDialog] = useState(false);

  // Fullscreen state
  const [isFullscreen, setIsFullscreen] = useState(false);

  // Ref for RhaiCodeEditor to enable cursor insertion
  const rhaiEditorRef = useRef<RhaiCodeEditorRef>(null);

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

  // Template handlers
  const handleTemplateSelect = useCallback((template: ExpressionTemplate) => {
    setSelectedTemplate(template);
    handleDialogClose();

    // If template has placeholders, show placeholder dialog, otherwise replace directly
    if (template.placeholders.length > 0) {
      setShowPlaceholderDialog(true);
    } else {
      setValue(template.rhaiCode); // Templates replace entire content
    }
  }, []);

  const handleTemplateInsert = useCallback((populatedCode: string) => {
    setValue(populatedCode);
    setShowPlaceholderDialog(false);
    setSelectedTemplate(null);
  }, []);

  const handlePlaceholderDialogClose = useCallback(() => {
    setShowPlaceholderDialog(false);
    setSelectedTemplate(null);
  }, []);

  const handleFullscreenToggle = useCallback(() => {
    setIsFullscreen((prev) => !prev);
  }, []);

  return (
    <>
      <Dialog open={open} onOpenChange={onClose}>
        <DialogContent
          size={isFullscreen ? "full" : "3xl"}
          onInteractOutside={(e) => e.preventDefault()}
          hideCloseButton>
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
                  {/* <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleDialogOpen("templates")}>
                    <CodeIcon className="h-4 w-4" />
                    {t("Templates")}
                  </Button> */}
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
                      <DropdownMenuTrigger asChild>
                        <Button variant="outline" size="sm">
                          <CircleIcon className="h-4 w-4" />
                          {t("Variables")}
                        </Button>
                      </DropdownMenuTrigger>
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
            {/* Raw Rhai Editor - Always Visible */}
            <div className="relative flex-1 border-b">
              <RhaiCodeEditor
                ref={rhaiEditorRef}
                className="h-full rounded-none bg-card/20 backdrop-blur-sm"
                placeholder={t("Enter expression...")}
                value={value}
                onChange={setValue}
                data-testid="value-editor-textarea"
                aria-label={t("Raw Expression Editor")}
                data-placeholder={t("Enter expression...")}
              />
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="absolute right-2 bottom-2 cursor-pointer p-1">
                    <QuestionIcon className="h-6 w-6" weight="thin" />
                  </div>
                </TooltipTrigger>
                <TooltipContent side="top" align="end">
                  <p className="text-sm">{t("Expression Editor Help")}</p>
                  <p className="mt-1 max-w-[200px] text-xs text-muted-foreground">
                    {t(
                      "Write Rhai expressions directly or use the visual builder below for assistance.",
                    )}
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
      {showDialog === "templates" && (
        <TemplateLibraryDialog
          open={showDialog === "templates"}
          onClose={handleDialogClose}
          onTemplateSelect={handleTemplateSelect}
        />
      )}
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
      <TemplatePlaceholderDialog
        open={showPlaceholderDialog}
        template={selectedTemplate}
        onClose={handlePlaceholderDialogClose}
        onInsert={handleTemplateInsert}
      />
    </>
  );
};

export default ValueEditorDialog;
