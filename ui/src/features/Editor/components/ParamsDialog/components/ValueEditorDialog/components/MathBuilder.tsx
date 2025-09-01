import { MathOperationsIcon, PlusIcon, TrashIcon } from "@phosphor-icons/react";
import { useCallback, useState, useEffect, useMemo } from "react";

import {
  Button,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import ExpressionInput from "./ExpressionInput";

type MathOperation = "+" | "-" | "*" | "/" | "%" | "**";
type MathFunction = "round" | "floor" | "ceil" | "abs" | "sqrt" | "min" | "max";

type ExpressionPart = {
  type: "value" | "operation" | "function";
  value: string;
  operation?: MathOperation;
  functionName?: MathFunction;
  precision?: number; // for round function
};

type Props = {
  onExpressionChange: (expression: string) => void;
};

const MathBuilder: React.FC<Props> = ({ onExpressionChange }) => {
  const t = useT();

  const [parts, setParts] = useState<ExpressionPart[]>([
    { type: "value", value: "" },
  ]);

  const operations = [
    { value: "+" as const, label: "Addition (+)", description: "Add values" },
    {
      value: "-" as const,
      label: "Subtraction (-)",
      description: "Subtract values",
    },
    {
      value: "*" as const,
      label: "Multiplication (*)",
      description: "Multiply values",
    },
    {
      value: "/" as const,
      label: "Division (/)",
      description: "Divide values",
    },
    {
      value: "%" as const,
      label: "Modulus (%)",
      description: "Remainder after division",
    },
    {
      value: "**" as const,
      label: "Power (**)",
      description: "Raise to power",
    },
  ];

  const functions = useMemo(
    () => [
      {
        value: "round" as const,
        label: "Round",
        description: "Round to decimal places",
        hasParameter: true,
      },
      {
        value: "floor" as const,
        label: "Floor",
        description: "Round down to integer",
        hasParameter: false,
      },
      {
        value: "ceil" as const,
        label: "Ceil",
        description: "Round up to integer",
        hasParameter: false,
      },
      {
        value: "abs" as const,
        label: "Absolute",
        description: "Absolute value",
        hasParameter: false,
      },
      {
        value: "sqrt" as const,
        label: "Square Root",
        description: "Square root",
        hasParameter: false,
      },
      {
        value: "min" as const,
        label: "Minimum",
        description: "Smallest of values",
        hasParameter: false,
      },
      {
        value: "max" as const,
        label: "Maximum",
        description: "Largest of values",
        hasParameter: false,
      },
    ],
    [],
  );

  // Generate expression for preview only - don't auto-insert
  const [currentExpression, setCurrentExpression] = useState("");

  useEffect(() => {
    // Generate expression for internal preview using original logic
    if (!parts.length || !parts[0].value) {
      setCurrentExpression("");
      return;
    }

    let expr = "";
    const validParts = parts.filter((part) => part.value.trim() !== "");

    if (validParts.length === 0) {
      setCurrentExpression("");
      return;
    }

    // Check if it's a single function application
    const firstPart = validParts[0];
    if (firstPart.type === "function" && validParts.length === 1) {
      const func = functions.find((f) => f.value === firstPart.functionName);
      if (func?.hasParameter && firstPart.precision !== undefined) {
        expr = `${firstPart.functionName}(${firstPart.value}, ${firstPart.precision})`;
      } else {
        expr = `${firstPart.functionName}(${firstPart.value})`;
      }
    } else {
      // Build regular mathematical expression
      expr = validParts
        .map((part, index) => {
          if (part.type === "function") {
            const func = functions.find((f) => f.value === part.functionName);
            if (func?.hasParameter && part.precision !== undefined) {
              return `${part.functionName}(${part.value}, ${part.precision})`;
            } else {
              return `${part.functionName}(${part.value})`;
            }
          } else if (part.type === "operation" && index > 0) {
            return ` ${part.operation} ${part.value}`;
          } else {
            return part.value;
          }
        })
        .join("");

      // Wrap in parentheses if it's a complex expression
      if (validParts.length > 1) {
        expr = `(${expr})`;
      }
    }

    setCurrentExpression(expr);
  }, [parts, functions]);

  const handleInsertExpression = useCallback(() => {
    if (currentExpression.trim()) {
      onExpressionChange(currentExpression);
    }
  }, [currentExpression, onExpressionChange]);

  const addOperation = useCallback(() => {
    setParts([...parts, { type: "operation", value: "", operation: "+" }]);
  }, [parts]);

  const addFunction = useCallback(() => {
    setParts([
      ...parts,
      { type: "function", value: "", functionName: "round", precision: 2 },
    ]);
  }, [parts]);

  const removePart = useCallback(
    (index: number) => {
      setParts(parts.filter((_, i) => i !== index));
    },
    [parts],
  );

  const updatePart = useCallback(
    (index: number, updates: Partial<ExpressionPart>) => {
      const newParts = [...parts];
      newParts[index] = { ...newParts[index], ...updates };
      setParts(newParts);
    },
    [parts],
  );

  // Common mathematical variables and values
  const commonValues = [
    "xmin",
    "xmax",
    "ymin",
    "ymax",
    "area",
    "length",
    "width",
    "height",
    "distance",
    "value",
    "count",
  ];

  return (
    <div className="flex size-full flex-col gap-4 p-4">
      <div className="flex-shrink-0">
        <h4 className="text-sm font-medium">{t("Mathematical Operations")}</h4>
        <p className="text-xs text-muted-foreground">
          {t("Build mathematical expressions with operations and functions")}
        </p>
      </div>

      <div className="space-y-4">
        {/* Examples section with better layout */}
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
          <div className="rounded border bg-muted/30 p-3 lg:col-span-2">
            <div className="mb-2 flex items-center gap-2 text-xs text-muted-foreground">
              <MathOperationsIcon className="h-4 w-4" />
              <span>{t("Examples:")} </span>
            </div>
            <div className="grid grid-cols-1 gap-1 md:grid-cols-2 lg:grid-cols-1">
              <code className="block text-xs">(xmin + xmax) / 2.0</code>
              <code className="block text-xs">round(area * 0.0001, 2)</code>
              <code className="block text-xs">
                sqrt(width ^ 2 + height ^ 2)
              </code>
            </div>
          </div>

          {/* Quick add buttons */}
          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">
              {t("Quick Add")}
            </Label>
            <div className="flex flex-col gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={addOperation}
                className="justify-start">
                <PlusIcon className="mr-2 h-3 w-3" />
                {t("Add Operation")}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={addFunction}
                className="justify-start">
                <PlusIcon className="mr-2 h-3 w-3" />
                {t("Add Function")}
              </Button>
            </div>
          </div>
        </div>

        <div className="space-y-3">
          {parts.map((part, index) => (
            <div key={index} className="rounded border p-4">
              <div className="mb-3 flex items-center justify-between">
                <span className="text-sm font-medium">
                  {index === 0
                    ? t("Value")
                    : part.type === "operation"
                      ? t("Operation")
                      : t("Function")}
                </span>
                {parts.length > 1 && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => removePart(index)}
                    className="h-6 w-6 p-0 text-destructive hover:text-destructive">
                    <TrashIcon className="h-3 w-3" />
                  </Button>
                )}
              </div>

              {part.type === "operation" && index > 0 && (
                <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
                  <div>
                    <Label className="text-xs">{t("Operation")}</Label>
                    <Select
                      value={part.operation}
                      onValueChange={(value) =>
                        updatePart(index, { operation: value as MathOperation })
                      }>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {operations.map((op) => (
                          <SelectItem key={op.value} value={op.value}>
                            <div>
                              <div className="text-sm">{op.label}</div>
                              <div className="text-xs text-muted-foreground">
                                {op.description}
                              </div>
                            </div>
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div>
                    <Label className="text-xs">{t("Value")}</Label>
                    <ExpressionInput
                      placeholder="2.0"
                      value={part.value}
                      onChange={(value) => updatePart(index, { value })}
                      className="text-sm"
                      label={t("Value")}
                      allowedExpressionTypes={[
                        "environment-variable",
                        "feature-attribute",
                        "math",
                      ]}
                    />
                  </div>
                </div>
              )}

              {part.type === "function" && (
                <div className="space-y-3">
                  <div>
                    <Label className="text-xs">{t("Function")}</Label>
                    <Select
                      value={part.functionName}
                      onValueChange={(value) =>
                        updatePart(index, {
                          functionName: value as MathFunction,
                        })
                      }>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {functions.map((func) => (
                          <SelectItem key={func.value} value={func.value}>
                            <div>
                              <div className="text-sm">{func.label}</div>
                              <div className="text-xs text-muted-foreground">
                                {func.description}
                              </div>
                            </div>
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
                    <div>
                      <Label className="text-xs">{t("Value")}</Label>
                      <ExpressionInput
                        placeholder="area"
                        value={part.value}
                        onChange={(value) => updatePart(index, { value })}
                        className="text-sm"
                        label={t("Value")}
                        allowedExpressionTypes={[
                          "environment-variable",
                          "feature-attribute",
                          "math",
                        ]}
                      />
                    </div>

                    {functions.find((f) => f.value === part.functionName)
                      ?.hasParameter && (
                      <div>
                        <Label className="text-xs">{t("Precision")}</Label>
                        <Input
                          type="number"
                          placeholder="2"
                          value={part.precision || ""}
                          onChange={(e) =>
                            updatePart(index, {
                              precision: parseInt(e.target.value) || 2,
                            })
                          }
                          className="text-sm"
                        />
                      </div>
                    )}
                  </div>
                </div>
              )}

              {part.type === "value" && (
                <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
                  <div>
                    <Label className="text-xs">{t("Value")}</Label>
                    <ExpressionInput
                      placeholder="xmin"
                      value={part.value}
                      onChange={(value) => updatePart(index, { value })}
                      className="text-sm"
                      label={t("Value")}
                      allowedExpressionTypes={[
                        "environment-variable",
                        "feature-attribute",
                        "math",
                      ]}
                    />
                  </div>
                  <div>
                    <Label className="text-xs text-muted-foreground">
                      {t("Common Values")}
                    </Label>
                    <div className="flex flex-wrap gap-1 pt-2">
                      {commonValues.slice(0, 8).map((val) => (
                        <button
                          key={val}
                          onClick={() => updatePart(index, { value: val })}
                          className="rounded bg-muted px-2 py-1 text-xs transition-colors hover:bg-accent">
                          {val}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>

        {/* Preview and Insert Section */}
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

export default MathBuilder;
