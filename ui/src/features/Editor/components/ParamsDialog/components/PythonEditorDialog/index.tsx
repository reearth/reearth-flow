import { Editor } from "@monaco-editor/react";
import { CodeIcon } from "@phosphor-icons/react";
import { useCallback, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
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

  const handleSubmit = useCallback(() => {
    if (!onValueSubmit) return;
    onValueSubmit(value);
    onClose();
  }, [value, onValueSubmit, onClose]);

  const handleEditorChange = useCallback((newValue: string | undefined) => {
    setValue(newValue || "");
  }, []);

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent size="3xl">
        <DialogHeader>
          <DialogTitle>
            <div className="flex items-center gap-2">
              <CodeIcon weight="thin" />
              {t("Python Editor")} -{" "}
              {fieldContext.schema.title ||
                fieldContext?.fieldName ||
                t("Unknown Field")}{" "}
              (Python Script)
            </div>
          </DialogTitle>
        </DialogHeader>

        <div className="flex h-[70vh] flex-col">
          {/* Editor */}
          <div className="flex-1 overflow-hidden rounded-md border border-border">
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

          {/* Footer with help text */}
          <div className="mt-4 rounded bg-muted/20 p-3 text-sm text-muted-foreground">
            <p>
              <strong>Available functions:</strong> get_geometry_type(),
              get_coordinates(), create_point(), create_polygon(),
              create_linestring()
            </p>
            <p>
              <strong>Available variables:</strong> properties, geometry,
              feature_id, attributes (alias for properties)
            </p>
          </div>

          {/* Submit Button */}
          <div className="mt-4 flex justify-end gap-2">
            <Button variant="outline" onClick={onClose}>
              {t("Cancel")}
            </Button>
            <Button onClick={handleSubmit}>{t("Apply")}</Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default PythonEditorDialog;
