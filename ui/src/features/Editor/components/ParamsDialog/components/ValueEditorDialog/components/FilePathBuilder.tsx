import { FolderIcon, FileIcon } from "@phosphor-icons/react";
import { useCallback, useState, useEffect } from "react";

import {
  Button,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import ExpressionInput from "./ExpressionInput";

type FilePathOperation =
  | "join_path"
  | "extract_filename"
  | "extract_filename_without_ext";

type Props = {
  onExpressionChange: (expression: string) => void;
};

const FilePathBuilder: React.FC<Props> = ({ onExpressionChange }) => {
  const t = useT();

  const [operation, setOperation] = useState<FilePathOperation>("join_path");
  const [path1, setPath1] = useState("");
  const [path2, setPath2] = useState("");

  const operations = [
    {
      value: "join_path" as const,
      label: t("Join Paths"),
      description: t("Combine directory and filename"),
      icon: <FolderIcon weight="thin" className="h-4 w-4" />,
      example: 'file::join_path("/output", "result.zip")',
    },
    {
      value: "extract_filename" as const,
      label: t("Extract Filename"),
      description: t("Get filename with extension"),
      icon: <FileIcon weight="thin" className="h-4 w-4" />,
      example: "file::extract_filename(path)",
    },
    {
      value: "extract_filename_without_ext" as const,
      label: t("Extract Name Only"),
      description: t("Get filename without extension"),
      icon: <FileIcon weight="thin" className="h-4 w-4" />,
      example: "file::extract_filename_without_ext(path)",
    },
  ];

  // Generate expression for preview only - don't auto-insert
  const [currentExpression, setCurrentExpression] = useState("");
  
  useEffect(() => {
    let expr = "";

    switch (operation) {
      case "join_path":
        if (path1 && path2) {
          expr = `file::join_path(${path1}, ${path2})`;
        } else if (path1) {
          // Show partial preview
          expr = `file::join_path(${path1}, "filename")`;
        }
        break;
      case "extract_filename":
        if (path1) {
          expr = `file::extract_filename(${path1})`;
        }
        break;
      case "extract_filename_without_ext":
        if (path1) {
          expr = `file::extract_filename_without_ext(${path1})`;
        }
        break;
    }

    setCurrentExpression(expr);
  }, [operation, path1, path2]);

  const handleInsertExpression = useCallback(() => {
    if (currentExpression.trim()) {
      onExpressionChange(currentExpression);
    }
  }, [currentExpression, onExpressionChange]);

  const selectedOperation = operations.find((op) => op.value === operation);

  return (
    <div className="flex size-full flex-col gap-4 p-4">
      <div className="flex-shrink-0">
        <h4 className="text-sm font-medium">{t("File Path Operations")}</h4>
        <p className="text-xs text-muted-foreground">
          {t("Build expressions for file path manipulation")}
        </p>
      </div>

      <div className="space-y-4">
        {/* Two-column layout for operation selection and example */}
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="operation-select" className="text-xs">
              {t("Operation")}
            </Label>
            <Select
              value={operation}
              onValueChange={(value) =>
                setOperation(value as FilePathOperation)
              }>
              <SelectTrigger id="operation-select">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {operations.map((op) => (
                  <SelectItem key={op.value} value={op.value}>
                    <div className="flex items-center gap-2">
                      {op.icon}
                      <div>
                        <div className="text-sm">{op.label}</div>
                        <div className="text-xs text-muted-foreground">
                          {op.description}
                        </div>
                      </div>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {selectedOperation && (
            <div className="rounded border bg-muted/30 p-3">
              <div className="mb-2 flex items-center gap-2 text-xs text-muted-foreground">
                {selectedOperation.icon}
                <span>{t("Example:")} </span>
              </div>
              <code className="text-xs break-all">
                {selectedOperation.example}
              </code>
            </div>
          )}
        </div>

        {/* Input fields with improved horizontal layout */}
        <div className="space-y-3">
          {operation === "join_path" && (
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
              <div className="space-y-2">
                <Label className="text-xs">{t("Directory Path")}</Label>
                <ExpressionInput
                  placeholder='"/output"'
                  value={path1}
                  onChange={setPath1}
                  className="text-sm"
                  label={t("Directory Path")}
                  allowedExpressionTypes={[
                    "environment-variable",
                    "feature-attribute",
                  ]}
                />
              </div>
              <div className="space-y-2">
                <Label className="text-xs">{t("Filename")}</Label>
                <ExpressionInput
                  placeholder='"result.zip"'
                  value={path2}
                  onChange={setPath2}
                  className="text-sm"
                  label={t("Filename")}
                  allowedExpressionTypes={[
                    "environment-variable",
                    "feature-attribute",
                  ]}
                />
              </div>
            </div>
          )}

          {(operation === "extract_filename" ||
            operation === "extract_filename_without_ext") && (
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
              <div className="space-y-2">
                <Label className="text-xs">{t("File Path")}</Label>
                <ExpressionInput
                  placeholder='"/path/to/file.txt"'
                  value={path1}
                  onChange={setPath1}
                  className="text-sm"
                  label={t("File Path")}
                  allowedExpressionTypes={[
                    "environment-variable",
                    "feature-attribute",
                  ]}
                />
              </div>
              {/* Empty column for consistent layout */}
              <div />
            </div>
          )}
        </div>
        
        {/* Preview and Insert Section */}
        {currentExpression && (
          <div className="mt-6 border-t pt-4">
            <div className="mb-3">
              <Label className="text-xs text-muted-foreground">{t("Preview")}</Label>
              <div className="mt-1 rounded border bg-muted/30 p-2 font-mono text-sm">
                {currentExpression}
              </div>
            </div>
            <Button onClick={handleInsertExpression} className="w-full">
              {t("Insert Expression")}
            </Button>
          </div>
        )}
      </div>
    </div>
  );
};

export default FilePathBuilder;
