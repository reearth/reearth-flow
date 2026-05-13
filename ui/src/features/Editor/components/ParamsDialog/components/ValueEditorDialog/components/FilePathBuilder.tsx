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

type FilePathOperation = "join_path" | "name" | "stem" | "extension" | "parent";

type Props = {
  onExpressionChange: (expression: string) => void;
};

const FilePathBuilder: React.FC<Props> = ({ onExpressionChange }) => {
  const t = useT();

  const [operation, setOperation] = useState<FilePathOperation>("join_path");
  const [basePath, setBasePath] = useState("");
  const [joinSuffix, setJoinSuffix] = useState("");

  const operations = [
    {
      value: "join_path" as const,
      label: t("Join Paths"),
      description: t("Combine base path with a sub-path or filename"),
      icon: <FolderIcon weight="thin" className="h-4 w-4" />,
      example: 'str(Url(env("basePath")) / "result.zip")',
    },
    {
      value: "name" as const,
      label: t("Extract Filename"),
      description: t("Get filename including extension"),
      icon: <FileIcon weight="thin" className="h-4 w-4" />,
      example: 'Url(value("path")).name()',
    },
    {
      value: "stem" as const,
      label: t("Extract Name Only"),
      description: t("Get filename without extension"),
      icon: <FileIcon weight="thin" className="h-4 w-4" />,
      example: 'Url(value("path")).stem()',
    },
    {
      value: "extension" as const,
      label: t("Extract Extension"),
      description: t("Get file extension without leading dot"),
      icon: <FileIcon weight="thin" className="h-4 w-4" />,
      example: 'Url(value("path")).extension()',
    },
    {
      value: "parent" as const,
      label: t("Parent Directory"),
      description: t("Get the parent directory as a Url"),
      icon: <FolderIcon weight="thin" className="h-4 w-4" />,
      example: 'Url(value("path")).parent()',
    },
  ];

  const [currentExpression, setCurrentExpression] = useState("");

  useEffect(() => {
    if (!basePath) {
      setCurrentExpression("");
      return;
    }

    let expr = "";

    switch (operation) {
      case "join_path": {
        const suffix = joinSuffix || '"filename"';
        expr = `str(Url(${basePath}) / ${suffix})`;
        break;
      }
      case "name":
        expr = `Url(${basePath}).name()`;
        break;
      case "stem":
        expr = `Url(${basePath}).stem()`;
        break;
      case "extension":
        expr = `Url(${basePath}).extension()`;
        break;
      case "parent":
        expr = `Url(${basePath}).parent()`;
        break;
    }

    setCurrentExpression(expr);
  }, [operation, basePath, joinSuffix]);

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
          {t("Build expressions for file path manipulation using the Url type")}
        </p>
      </div>

      <div className="space-y-4">
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

        <div className="space-y-3">
          <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
            <div className="space-y-2">
              <Label className="text-xs">{t("Base Path")}</Label>
              <ExpressionInput
                placeholder='env("basePath")'
                value={basePath}
                onChange={setBasePath}
                className="text-sm"
                label={t("Base Path")}
                allowedExpressionTypes={[
                  "environment-variable",
                  "feature-attribute",
                ]}
              />
            </div>
            {operation === "join_path" && (
              <div className="space-y-2">
                <Label className="text-xs">{t("Suffix / Filename")}</Label>
                <ExpressionInput
                  placeholder='"result.zip"'
                  value={joinSuffix}
                  onChange={setJoinSuffix}
                  className="text-sm"
                  label={t("Suffix")}
                  allowedExpressionTypes={[
                    "environment-variable",
                    "feature-attribute",
                  ]}
                />
              </div>
            )}
            {operation !== "join_path" && <div />}
          </div>

          {operation === "join_path" && (
            <div className="rounded border bg-muted/30 p-3 text-xs text-muted-foreground">
              <div className="space-y-1">
                <div>
                  •{" "}
                  {t(
                    "The / operator joins path segments — use str() to convert back to a string",
                  )}
                </div>
                <div>• {t("Chain multiple: Url(base) / sub / file")}</div>
              </div>
            </div>
          )}
        </div>

        {currentExpression && (
          <div className="mt-6 border-t pt-4">
            <div className="mb-3">
              <Label className="text-xs text-muted-foreground">
                {t("Preview")}
              </Label>
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
