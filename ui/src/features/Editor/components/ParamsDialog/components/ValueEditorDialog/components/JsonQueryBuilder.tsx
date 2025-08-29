import { DatabaseIcon, CheckIcon } from "@phosphor-icons/react";
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

type JsonOperation = "find_value_by_json_path" | "exists_value_by_json_path";

type Props = {
  onExpressionChange: (expression: string) => void;
};

const JsonQueryBuilder: React.FC<Props> = ({ onExpressionChange }) => {
  const t = useT();

  const [operation, setOperation] = useState<JsonOperation>(
    "find_value_by_json_path",
  );
  const [jsonContent, setJsonContent] = useState("");
  const [jsonPath, setJsonPath] = useState("");

  const operations = [
    {
      value: "find_value_by_json_path" as const,
      label: t("Find JSON Value"),
      description: t("Extract value from JSON using JSONPath"),
      icon: <DatabaseIcon weight="thin" className="h-4 w-4" />,
      example: 'json::find_value_by_json_path(content, "$.data.name")',
    },
    {
      value: "exists_value_by_json_path" as const,
      label: t("Check JSON Path Exists"),
      description: t("Check if JSONPath exists in JSON"),
      icon: <CheckIcon weight="thin" className="h-4 w-4" />,
      example: 'json::exists_value_by_json_path(content, "$.data")',
    },
  ];

  // Generate expression for preview only - don't auto-insert
  const [currentExpression, setCurrentExpression] = useState("");
  
  useEffect(() => {
    let expr = "";

    if (jsonContent && jsonPath) {
      switch (operation) {
        case "find_value_by_json_path":
          expr = `json::find_value_by_json_path(${jsonContent}, ${jsonPath})`;
          break;
        case "exists_value_by_json_path":
          expr = `json::exists_value_by_json_path(${jsonContent}, ${jsonPath})`;
          break;
      }
    }

    setCurrentExpression(expr);
  }, [operation, jsonContent, jsonPath]);

  const handleInsertExpression = useCallback(() => {
    if (currentExpression.trim()) {
      onExpressionChange(currentExpression);
    }
  }, [currentExpression, onExpressionChange]);

  const selectedOperation = operations.find((op) => op.value === operation);

  return (
    <div className="flex size-full flex-col gap-4 p-4">
      <div className="flex-shrink-0">
        <h4 className="text-sm font-medium">{t("JSON Query Operations")}</h4>
        <p className="text-xs text-muted-foreground">
          {t("Query and validate JSON data using JSONPath expressions")}
        </p>
      </div>

      <div className="space-y-4">
        {/* Operation selection and example */}
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="json-operation-select" className="text-xs">
              {t("Operation")}
            </Label>
            <Select
              value={operation}
              onValueChange={(value) => setOperation(value as JsonOperation)}>
              <SelectTrigger id="json-operation-select">
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
                <span>{t("Example:")}</span>
              </div>
              <code className="text-xs break-all">
                {selectedOperation.example}
              </code>
            </div>
          )}
        </div>

        {/* Input fields */}
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <div className="space-y-2">
            <Label className="text-xs">{t("JSON Content")}</Label>
            <ExpressionInput
              placeholder='env.get("__value").jsonData'
              value={jsonContent}
              onChange={setJsonContent}
              className="text-sm"
              label={t("JSON Content")}
              allowedExpressionTypes={[
                "environment-variable",
                "feature-attribute",
              ]}
            />
          </div>
          <div className="space-y-2">
            <Label className="text-xs">{t("JSONPath Query")}</Label>
            <ExpressionInput
              placeholder='"$.data.items[0].name"'
              value={jsonPath}
              onChange={setJsonPath}
              className="text-sm"
              label={t("JSONPath Query")}
              allowedExpressionTypes={[
                "environment-variable",
                "feature-attribute",
              ]}
            />
          </div>
        </div>

        {/* JSONPath Help */}
        <div className="rounded border-l-4 border-blue-200 bg-blue-50 p-3 dark:border-blue-800 dark:bg-blue-950/20">
          <h5 className="mb-1 text-xs font-medium text-blue-900 dark:text-blue-200">
            {t("JSONPath Examples:")}
          </h5>
          <ul className="space-y-1 text-xs text-blue-800 dark:text-blue-300">
            <li>
              <code>"$.data"</code> - Root data object
            </li>
            <li>
              <code>"$.items[0]"</code> - First item in items array
            </li>
            <li>
              <code>"$.user.name"</code> - Nested property access
            </li>
            <li>
              <code>"$..name"</code> - All name properties recursively
            </li>
          </ul>
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

export default JsonQueryBuilder;
