import {
  PencilLineIcon,
  CornersInIcon,
  CornersOutIcon,
  FileIcon,
  CircleIcon,
} from "@phosphor-icons/react";
import { useCallback, useRef, useState } from "react";

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
  TextArea,
  CmsLogo,
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
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
} from "../ValueEditorDialog/components/FlowExprCodeEditor";
import { AutocompleteSuggestion } from "../ValueEditorDialog/components/flowExprConstants";

export type CodeValue = {
  type: "flowExpr" | "string";
  value: string;
};

type DialogMode = "assets" | "cms" | undefined;

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
    | string[]
    | undefined;
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
  const [showDialog, setShowDialog] = useState<DialogMode>(undefined);

  const editorRef = useRef<FlowExprCodeEditorRef>(null);

  const [currentProject] = useCurrentProject();
  const { useGetWorkflowVariables } = useWorkflowVariables();
  const { workflowVariables } = useGetWorkflowVariables(currentProject?.id);

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

  const handleAssetSelect = useCallback(
    (asset: Asset) => {
      // In FlowExpr, wrap asset URLs with Url(...) in expression mode
      const snippet =
        codeType === "flowExpr" ? `Url("${asset.url}")` : asset.url;
      insertAtCursor(snippet);
      setShowDialog(undefined);
    },
    [codeType, insertAtCursor],
  );

  const handleCmsItemValue = useCallback(
    (url: string) => {
      const snippet = codeType === "flowExpr" ? `Url("${url}")` : url;
      insertAtCursor(snippet);
      setShowDialog(undefined);
    },
    [codeType, insertAtCursor],
  );

  const handleVariableSelect = useCallback(
    (variableName: string) => {
      insertAtCursor(`env["${variableName}"]`);
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
                  {t("FlowExpr Editor")} — {fieldLabel}
                </div>
                <div className="flex flex-1 items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowDialog("assets")}>
                    <FileIcon className="h-4 w-4" />
                    {t("Workspace Assets")}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowDialog("cms")}>
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
                            onClick={() => handleVariableSelect(variable.name)}
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
            {/* Mode toggle — only shown when more than one type is allowed */}
            {flowExprAllowed && stringAllowed && (
              <div className="flex shrink-0 gap-1 border-b px-4 py-2">
                <TabsList className="flex gap-2">
                  <TabsTrigger value="flowExpr">{t("Expression")}</TabsTrigger>
                  <TabsTrigger value="string">
                    {t("Literal string")}
                  </TabsTrigger>
                </TabsList>
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
                placeholder={t('e.g. Url(env.get("BASE_DIR")) / "filename"')}
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
                    'FlowExpr: env["VAR"], attributes["attr"], Url(...), math.sin(...). API is still stabilising.',
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

      {showDialog === "assets" && (
        <AssetsDialog
          onDialogClose={() => setShowDialog(undefined)}
          onAssetSelect={handleAssetSelect}
        />
      )}
      {showDialog === "cms" && (
        <CmsIntegrationDialog
          onDialogClose={() => setShowDialog(undefined)}
          onCmsItemValue={handleCmsItemValue}
        />
      )}
    </>
  );
};

export default FlowExprEditorDialog;
