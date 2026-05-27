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

export type CodeValue = {
  type: "flowExpr" | "string";
  value: string;
};

type DialogMode = "assets" | "cms" | undefined;

type Props = {
  open: boolean;
  fieldContext: FieldContext;
  onClose: () => void;
  onValueSubmit?: (value: CodeValue) => void;
};

const FlowExprEditorDialog: React.FC<Props> = ({
  open,
  fieldContext,
  onClose,
  onValueSubmit,
}) => {
  const t = useT();

  const initialCode = fieldContext.value as CodeValue | undefined;
  const [codeType, setCodeType] = useState<"flowExpr" | "string">(
    initialCode?.type ?? "flowExpr",
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
      // FlowExpr uses env("VAR") — Rhai uses env.get("VAR")
      const snippet =
        codeType === "flowExpr" ? `env("${variableName}")` : variableName;
      insertAtCursor(snippet);
    },
    [codeType, insertAtCursor],
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

          <div
            className={`flex flex-col ${isFullscreen ? "h-[calc(100vh-52px)]" : "h-[70vh]"}`}>
            {/* Mode toggle */}
            <div className="flex shrink-0 gap-1 border-b px-4 py-2">
              <button
                type="button"
                onClick={() => setCodeType("flowExpr")}
                className={`rounded px-3 py-1 text-sm font-medium transition-colors ${
                  codeType === "flowExpr"
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:text-foreground"
                }`}>
                {t("Expression")}
              </button>
              <button
                type="button"
                onClick={() => setCodeType("string")}
                className={`rounded px-3 py-1 text-sm font-medium transition-colors ${
                  codeType === "string"
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:text-foreground"
                }`}>
                {t("Literal string")}
              </button>
            </div>

            {/* Editor area */}
            <div className="relative flex-1 overflow-hidden border-b">
              {codeType === "flowExpr" ? (
                <FlowExprCodeEditor
                  ref={editorRef}
                  className="size-full"
                  value={codeValue}
                  onChange={setCodeValue}
                  placeholder={t(
                    'e.g. Url(env("BASE_DIR")) / value("filename")',
                  )}
                />
              ) : (
                <TextArea
                  className="size-full resize-none rounded-none border-0 focus-visible:ring-0"
                  value={codeValue}
                  onChange={(e) => setCodeValue(e.target.value)}
                  placeholder={t("Enter a literal string value")}
                  spellCheck={false}
                />
              )}
            </div>

            {/* Hint bar */}
            <div className="shrink-0 bg-muted/20 px-4 py-2 text-xs text-muted-foreground">
              {codeType === "flowExpr" ? (
                <span>
                  {t(
                    'FlowExpr: use value("attr"), env("VAR"), Url(...), math::sqrt(...)',
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
          </div>
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
