import { Editor } from "@monaco-editor/react";
import { CodeIcon, CornersInIcon, CornersOutIcon } from "@phosphor-icons/react";
import { useCallback, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  IconButton,
} from "@flow/components";
import type { EditorContext as FieldContext } from "@flow/components/SchemaForm";
import { useT } from "@flow/lib/i18n";

export type CodeValue = {
  type: "flowExpr" | "string";
  value: string;
};

type Props = {
  open: boolean;
  fieldContext: FieldContext;
  onClose: () => void;
  onValueSubmit?: (value: CodeValue) => void;
};

/**
 * The field holds `{ type, value }`, not a bare string — every `format: "code"`
 * field in the engine's schema requires both keys. Reading `fieldContext.value`
 * straight into the editor put the whole object in front of the user, and
 * submitting the editor's string wrote a string where the schema wants the
 * object, so the script failed validation and was dropped on reload.
 */
const toCodeValue = (value: unknown): CodeValue => {
  if (typeof value === "string") return { type: "string", value };
  if (value && typeof value === "object") {
    const record = value as Partial<CodeValue>;
    return {
      // Keep the type the field already had; a script written as an expression
      // stays one.
      type: record.type === "flowExpr" ? "flowExpr" : "string",
      value: typeof record.value === "string" ? record.value : "",
    };
  }
  return { type: "string", value: "" };
};

const PythonEditorDialog: React.FC<Props> = ({
  open,
  fieldContext,
  onClose,
  onValueSubmit,
}) => {
  const t = useT();
  const initial = toCodeValue(fieldContext.value);
  const [value, setValue] = useState(initial.value);
  const [isFullscreen, setIsFullscreen] = useState(false);

  const handleSubmit = useCallback(() => {
    if (!onValueSubmit) return;
    onValueSubmit({ type: initial.type, value });
    onClose();
  }, [initial.type, value, onValueSubmit, onClose]);

  const handleEditorChange = useCallback((newValue: string | undefined) => {
    setValue(newValue || "");
  }, []);

  const handleFullscreenToggle = useCallback(() => {
    setIsFullscreen((prev) => !prev);
  }, []);

  return (
    <Dialog open={open} disablePointerDismissal onOpenChange={onClose}>
      <DialogContent size={isFullscreen ? "full" : "3xl"} hideCloseButton>
        <DialogHeader>
          <DialogTitle className="relative flex items-center justify-between">
            <div className="flex items-center gap-2">
              <CodeIcon weight="thin" />
              {t("Python Editor")} -{" "}
              {fieldContext.schema.title ||
                fieldContext?.fieldName ||
                t("Unknown Field")}{" "}
              (Python Script)
            </div>
            <IconButton
              className="absolute top-2 right-2 rounded-[4px]"
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
          {/* Editor */}
          <div className="flex-1 overflow-hidden">
            <Editor
              height="100%"
              defaultLanguage="python"
              value={value}
              onChange={handleEditorChange}
              theme="vs-dark"
              options={{
                minimap: { enabled: false },
                fontSize: 14,
                lineNumbers: "on",
                roundedSelection: false,
                scrollBeyondLastLine: false,
                automaticLayout: true,
                tabSize: 4,
                wordWrap: "on",
                suggest: {
                  showKeywords: true,
                  showSnippets: true,
                },
                quickSuggestions: {
                  other: true,
                  comments: true,
                  strings: true,
                },
              }}
            />
          </div>
          <div className="border-b bg-muted/20 p-4 text-sm text-muted-foreground">
            <p>
              <strong>{t("Available functions:")}</strong> get_geometry_type(),
              get_coordinates(), create_point(), create_polygon(),
              create_linestring()
            </p>
            <p>
              <strong>{t("Available variables:")}</strong> properties, geometry,
              feature_id, attributes ({t("alias for properties")})
            </p>
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
  );
};

export default PythonEditorDialog;
