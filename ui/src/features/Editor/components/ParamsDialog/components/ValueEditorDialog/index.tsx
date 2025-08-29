import {
  PencilLineIcon,
  CaretLeftIcon,
  CircleIcon,
  ArchiveIcon,
  DatabaseIcon,
  CaretDownIcon,
  CaretUpIcon,
  WrenchIcon,
  CodeIcon,
} from "@phosphor-icons/react";
import { QuestionIcon } from "@phosphor-icons/react/dist/ssr";
import { useCallback, useState, useRef } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  ScrollArea,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import CmsIntegrationDialog from "@flow/features/CmsIntegrationDialog";
import { useProjectVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import { Asset } from "@flow/types";

import { FieldContext } from "../../utils/fieldUtils";

import ConditionalBuilder from "./components/ConditionalBuilder";
import EnvironmentVariableBuilder from "./components/EnvironmentVariableBuilder";
import ExpressionTypePicker, {
  type ExpressionType,
} from "./components/ExpressionTypePicker";
import FeatureAttributeBuilder from "./components/FeatureAttributeBuilder";
import FilePathBuilder from "./components/FilePathBuilder";
import JsonQueryBuilder from "./components/JsonQueryBuilder";
import MathBuilder from "./components/MathBuilder";
import RhaiCodeEditor, { type RhaiCodeEditorRef } from "./components/RhaiCodeEditor";
import { TemplateLibraryDialog, TemplatePlaceholderDialog, type ExpressionTemplate } from "./templates";

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

  // Track selected expression type for Simple Builder
  const [selectedExpressionType, setSelectedExpressionType] =
    useState<ExpressionType | null>(null);

  // Track Simple Builder panel visibility
  const [simpleBuilderOpen, setSimpleBuilderOpen] = useState(
    !value || value.trim() === "",
  );

  // Template-related state
  const [selectedTemplate, setSelectedTemplate] = useState<ExpressionTemplate | null>(null);
  const [showPlaceholderDialog, setShowPlaceholderDialog] = useState(false);


  // Ref for RhaiCodeEditor to enable cursor insertion
  const rhaiEditorRef = useRef<RhaiCodeEditorRef>(null);

  const [currentProject] = useCurrentProject();

  const { useGetProjectVariables } = useProjectVariables();

  const { projectVariables } = useGetProjectVariables(currentProject?.id);

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

  const handleProjectVariableSet = useCallback((variable: any) => {
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

  const handleExpressionTypeSelect = useCallback((type: ExpressionType) => {
    setSelectedExpressionType(type);
  }, []);

  const handleExpressionBuilderChange = useCallback((expression: string) => {
    // Insert at cursor position instead of replacing entire content
    if (rhaiEditorRef.current) {
      rhaiEditorRef.current.insertAtCursor(expression);
    } else {
      // Fallback to setValue if ref is not available
      setValue(expression);
    }
  }, []);

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

  return (
    <>
      <Dialog open={open} onOpenChange={onClose}>
        <DialogContent size="3xl">
          <DialogHeader>
            <DialogTitle>
              <div className="flex items-center gap-2">
                <PencilLineIcon weight="thin" />
                {t("Value Editor")} -{" "}
                {fieldContext.schema.title ||
                  fieldContext?.fieldName ||
                  t("Unknown Field")}{" "}
                {fieldType ? `(${fieldType})` : ""}
              </div>
            </DialogTitle>
          </DialogHeader>

          <div className="flex h-[600px] flex-col">
            {/* Action Bar */}
            <div className="flex items-center justify-between border-b p-3">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium text-muted-foreground">
                  {t("Expression Editor")}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleDialogOpen("templates")}>
                  <CodeIcon className="mr-2 h-4 w-4" />
                  {t("Templates")}
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleDialogOpen("assets")}>
                  <ArchiveIcon className="mr-2 h-4 w-4" />
                  {t("Asset")}
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleDialogOpen("cms")}>
                  <DatabaseIcon className="mr-2 h-4 w-4" />
                  {t("CMS")}
                </Button>
                {projectVariables && projectVariables.length > 0 && (
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="outline" size="sm">
                        <CircleIcon className="mr-2 h-4 w-4" />
                        {t("Variables")}
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end" className="w-64">
                      {projectVariables.map((variable) => (
                        <DropdownMenuItem
                          key={variable.id}
                          onClick={() => handleProjectVariableSet(variable)}
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
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="outline" size="sm" className="px-2">
                      <QuestionIcon className="h-4 w-4" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="bottom">
                    <p className="text-sm">{t("Expression Editor Help")}</p>
                    <p className="mt-1 max-w-[200px] text-xs text-muted-foreground">
                      {t(
                        "Write Rhai expressions directly or use the visual builder below for assistance.",
                      )}
                    </p>
                  </TooltipContent>
                </Tooltip>
              </div>
            </div>

            {/* Raw Rhai Editor - Always Visible */}
            <div className="flex-1 border-b">
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
            </div>

            {/* Collapsible Simple Builder Panel */}
            <Collapsible
              open={simpleBuilderOpen}
              onOpenChange={setSimpleBuilderOpen}>
              <div className="border-b">
                <CollapsibleTrigger asChild>
                  <Button
                    variant="ghost"
                    className="flex h-12 w-full items-center justify-between rounded-none px-4 hover:bg-accent/50">
                    <div className="flex items-center gap-2">
                      <WrenchIcon className="h-4 w-4" />
                      <span className="text-sm font-medium">
                        {t("Simple Builder")}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {t("Visual expression builder")}
                      </span>
                    </div>
                    {simpleBuilderOpen ? (
                      <CaretUpIcon className="h-4 w-4" />
                    ) : (
                      <CaretDownIcon className="h-4 w-4" />
                    )}
                  </Button>
                </CollapsibleTrigger>
              </div>

              <CollapsibleContent className="border-b">
                <div className="flex h-[350px] flex-col">
                  {/* Simple Builder Navigation */}
                  {selectedExpressionType && (
                    <div className="px-2 pt-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setSelectedExpressionType(null)}
                        className="h-8 gap-1 px-2">
                        <CaretLeftIcon className="h-4 w-4" />
                      </Button>
                    </div>
                  )}

                  {/* Simple Builder Content */}
                  <ScrollArea className="flex-1">
                    <div className="px-4 pt-4">
                      {!selectedExpressionType ? (
                        <ExpressionTypePicker
                          onTypeSelect={handleExpressionTypeSelect}
                        />
                      ) : (
                        <div className="min-h-0">
                          {selectedExpressionType === "file-path" && (
                            <FilePathBuilder
                              onExpressionChange={handleExpressionBuilderChange}
                            />
                          )}
                          {selectedExpressionType === "feature-attribute" && (
                            <FeatureAttributeBuilder
                              onExpressionChange={handleExpressionBuilderChange}
                            />
                          )}
                          {selectedExpressionType === "conditional" && (
                            <ConditionalBuilder
                              onExpressionChange={handleExpressionBuilderChange}
                            />
                          )}
                          {selectedExpressionType === "math" && (
                            <MathBuilder
                              onExpressionChange={handleExpressionBuilderChange}
                            />
                          )}
                          {selectedExpressionType ===
                            "environment-variable" && (
                            <EnvironmentVariableBuilder
                              onExpressionChange={handleExpressionBuilderChange}
                            />
                          )}
                          {selectedExpressionType === "json-query" && (
                            <JsonQueryBuilder
                              onExpressionChange={handleExpressionBuilderChange}
                            />
                          )}
                          {![
                            "file-path",
                            "feature-attribute",
                            "conditional",
                            "math",
                            "environment-variable",
                            "json-query",
                          ].includes(selectedExpressionType) && (
                            <div className="flex flex-1 flex-col items-center justify-center p-8 text-center text-muted-foreground">
                              <p className="mb-4">
                                {t("Selected:")} {selectedExpressionType}
                              </p>
                              <div className="text-sm">
                                {t(
                                  "Expression builder for {{type}} will go here",
                                  {
                                    type: selectedExpressionType,
                                  },
                                )}
                              </div>
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  </ScrollArea>
                </div>
              </CollapsibleContent>
            </Collapsible>

            {/* Submit Button */}
            <div className="flex justify-end gap-2 p-3">
              <Button onClick={handleSubmit}>{t("Submit")}</Button>
            </div>
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
          onAssetDoubleClick={handleAssetDoubleClick}
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
