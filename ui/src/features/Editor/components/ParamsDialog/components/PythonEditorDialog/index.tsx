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
import { useT } from "@flow/lib/i18n";

import { FieldContext } from "../../utils/fieldUtils";

type Props = {
  open: boolean;
  fieldContext: FieldContext;
  onClose: () => void;
  onValueSubmit?: (value: any) => void;
};

const PythonEditorDialog: React.FC<Props> = ({
  open,
  fieldContext,
  onClose,
  onValueSubmit,
}) => {
  const t = useT();
  const [value, setValue] = useState(fieldContext.value || "");
  const [isFullscreen, setIsFullscreen] = useState(false);

  const handleSubmit = useCallback(() => {
    if (!onValueSubmit) return;
    onValueSubmit(value);
    onClose();
  }, [value, onValueSubmit, onClose]);

  const handleEditorChange = useCallback((newValue: string | undefined) => {
    setValue(newValue || "");
  }, []);

  const handleFullscreenToggle = useCallback(() => {
    setIsFullscreen((prev) => !prev);
  }, []);

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent
        size={isFullscreen ? "full" : "3xl"}
        onInteractOutside={(e) => e.preventDefault()}
        hideCloseButton
        // className={isFullscreen ? "fixed inset-0 max-h-screen max-w-screen" : ""}
      >
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
