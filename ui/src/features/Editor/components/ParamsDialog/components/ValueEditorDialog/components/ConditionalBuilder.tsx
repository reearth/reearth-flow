import { GitBranchIcon, TrashIcon } from "@phosphor-icons/react";
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

type ConditionOperator =
  | "=="
  | "!="
  | ">"
  | "<"
  | ">="
  | "<="
  | "contains"
  | "starts_with"
  | "ends_with"
  | "is_empty"
  | "is_not_empty";

type ConditionalType = "if_else" | "ternary";

type Condition = {
  left: string;
  operator: ConditionOperator;
  right: string;
};

type Props = {
  onExpressionChange: (expression: string) => void;
};

const ConditionalBuilder: React.FC<Props> = ({ onExpressionChange }) => {
  const t = useT();

  const [conditionalType, setConditionalType] =
    useState<ConditionalType>("if_else");
  const [conditions, setConditions] = useState<Condition[]>([
    { left: "", operator: "==", right: "" },
  ]);
  const [trueResult, setTrueResult] = useState("");
  const [elseResult, setElseResult] = useState("");

  const conditionalTypes = [
    {
      value: "if_else" as const,
      label: t("If-Then-Else"),
      description: t("Standard if-then-else conditional"),
      example: 'if condition { "result" } else { "default" }',
    },
    {
      value: "ternary" as const,
      label: t("Ternary Operator"),
      description: t("Compact ternary expression"),
      example: 'city == "Tokyo" ? "JP" : "Other"',
    },
  ];

  const operators = [
    { value: "==" as const, label: "equals (==)", description: "Equal to" },
    {
      value: "!=" as const,
      label: "not equals (!=)",
      description: "Not equal to",
    },
    {
      value: ">" as const,
      label: "greater than (>)",
      description: "Greater than",
    },
    { value: "<" as const, label: "less than (<)", description: "Less than" },
    {
      value: ">=" as const,
      label: "greater or equal (>=)",
      description: "Greater than or equal",
    },
    {
      value: "<=" as const,
      label: "less or equal (<=)",
      description: "Less than or equal",
    },
    {
      value: "contains" as const,
      label: "contains",
      description: "String contains",
    },
    {
      value: "starts_with" as const,
      label: "starts with",
      description: "String starts with",
    },
    {
      value: "ends_with" as const,
      label: "ends with",
      description: "String ends with",
    },
    {
      value: "is_empty" as const,
      label: "is empty",
      description: "Value is empty",
    },
    {
      value: "is_not_empty" as const,
      label: "is not empty",
      description: "Value is not empty",
    },
  ];

  const generateExpression = useCallback(() => {
    if (!conditions.length || !conditions[0].left) {
      onExpressionChange("");
      return;
    }

    let expr = "";

    switch (conditionalType) {
      case "if_else": {
        const condition = conditions[0];
        let conditionStr = "";

        if (["is_empty", "is_not_empty"].includes(condition.operator)) {
          conditionStr =
            condition.operator === "is_empty"
              ? `${condition.left}.is_empty()`
              : `!${condition.left}.is_empty()`;
        } else if (
          ["contains", "starts_with", "ends_with"].includes(condition.operator)
        ) {
          conditionStr = `${condition.left}.${condition.operator}(${condition.right})`;
        } else {
          conditionStr = `${condition.left} ${condition.operator} ${condition.right}`;
        }

        expr = `if ${conditionStr} { ${trueResult} } else { ${elseResult} }`;
        break;
      }

      case "ternary": {
        const ternaryCondition = conditions[0];
        let ternaryConditionStr = "";

        if (["is_empty", "is_not_empty"].includes(ternaryCondition.operator)) {
          ternaryConditionStr =
            ternaryCondition.operator === "is_empty"
              ? `${ternaryCondition.left}.is_empty()`
              : `!${ternaryCondition.left}.is_empty()`;
        } else if (
          ["contains", "starts_with", "ends_with"].includes(
            ternaryCondition.operator,
          )
        ) {
          ternaryConditionStr = `${ternaryCondition.left}.${ternaryCondition.operator}(${ternaryCondition.right})`;
        } else {
          ternaryConditionStr = `${ternaryCondition.left} ${ternaryCondition.operator} ${ternaryCondition.right}`;
        }

        expr = `${ternaryConditionStr} ? ${trueResult} : ${elseResult}`;
        break;
      }
    }

    onExpressionChange(expr);
  }, [conditionalType, conditions, trueResult, elseResult, onExpressionChange]);

  useEffect(() => {
    generateExpression();
  }, [generateExpression]);

  const removeCondition = useCallback(
    (index: number) => {
      setConditions(conditions.filter((_, i) => i !== index));
    },
    [conditions],
  );

  const updateCondition = useCallback(
    (index: number, field: keyof Condition, value: string) => {
      const newConditions = [...conditions];
      newConditions[index] = { ...newConditions[index], [field]: value };
      setConditions(newConditions);
    },
    [conditions],
  );

  const selectedType = conditionalTypes.find(
    (type) => type.value === conditionalType,
  );

  return (
    <div className="flex size-full flex-col gap-4 p-4">
      <div className="flex-shrink-0">
        <h4 className="text-sm font-medium">{t("Conditional Logic")}</h4>
        <p className="text-xs text-muted-foreground">
          {t("Build if-then-else expressions for dynamic behavior")}
        </p>
      </div>

      <div className="space-y-4">
        {/* Two-column layout for type selection and example */}
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="conditional-type-select" className="text-xs">
              {t("Conditional Type")}
            </Label>
            <Select
              value={conditionalType}
              onValueChange={(value) =>
                setConditionalType(value as ConditionalType)
              }>
              <SelectTrigger id="conditional-type-select">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {conditionalTypes.map((type) => (
                  <SelectItem key={type.value} value={type.value}>
                    <div>
                      <div className="text-sm">{type.label}</div>
                      <div className="text-xs text-muted-foreground">
                        {type.description}
                      </div>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {selectedType && (
            <div className="rounded border bg-muted/30 p-3">
              <div className="mb-2 flex items-center gap-2 text-xs text-muted-foreground">
                <GitBranchIcon className="h-4 w-4" />
                <span>{t("Example:")} </span>
              </div>
              <code className="text-xs break-all">{selectedType.example}</code>
            </div>
          )}
        </div>

        <div className="space-y-4">
          {/* Condition section */}
          {conditions.map((condition, index) => (
            <div key={index} className="rounded border p-4">
              <div className="mb-3 flex items-center justify-between">
                <span className="text-sm font-medium">
                  {index === 0 ? t("Condition") : t("Else If")}
                </span>
                {conditions.length > 1 && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => removeCondition(index)}
                    className="h-6 w-6 p-0 text-destructive hover:text-destructive">
                    <TrashIcon className="h-3 w-3" />
                  </Button>
                )}
              </div>

              <div className="grid grid-cols-1 gap-3 lg:grid-cols-3">
                <div>
                  <Label className="text-xs">{t("Left Value")}</Label>
                  <ExpressionInput
                    placeholder='env.get("cityCode")'
                    value={condition.left}
                    onChange={(value) => updateCondition(index, "left", value)}
                    className="text-sm"
                    label={t("Left Value")}
                    allowedExpressionTypes={[
                      "feature-attribute",
                      "environment-variable",
                      "math",
                    ]}
                  />
                </div>

                <div>
                  <Label className="text-xs">{t("Operator")}</Label>
                  <Select
                    value={condition.operator}
                    onValueChange={(value) =>
                      updateCondition(
                        index,
                        "operator",
                        value as ConditionOperator,
                      )
                    }>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {operators.map((op) => (
                        <SelectItem key={op.value} value={op.value}>
                          <div>
                            <div className="text-xs">{op.label}</div>
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div>
                  <Label className="text-xs">
                    {["is_empty", "is_not_empty"].includes(condition.operator)
                      ? t("(not used)")
                      : t("Right Value")}
                  </Label>
                  <ExpressionInput
                    placeholder='"Tokyo"'
                    value={condition.right}
                    onChange={(value) => updateCondition(index, "right", value)}
                    className="text-sm"
                    label={t("Right Value")}
                    disabled={["is_empty", "is_not_empty"].includes(
                      condition.operator,
                    )}
                    allowedExpressionTypes={[
                      "feature-attribute",
                      "environment-variable",
                      "math",
                    ]}
                  />
                </div>
              </div>
            </div>
          ))}

          {/* Results section with horizontal layout */}
          <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
            <div className="space-y-2">
              <Label className="text-xs">{t("True Result")}</Label>
              <ExpressionInput
                placeholder='"success"'
                value={trueResult}
                onChange={setTrueResult}
                className="text-sm"
                label={t("True Result")}
                allowedExpressionTypes={[
                  "file-path",
                  "environment-variable",
                  "feature-attribute",
                ]}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="else-result" className="text-xs">
                {t("Else Result")}
              </Label>
              <ExpressionInput
                placeholder='"default"'
                value={elseResult}
                onChange={setElseResult}
                className="text-sm"
                label={t("Else Result")}
                allowedExpressionTypes={[
                  "file-path",
                  "environment-variable",
                  "feature-attribute",
                ]}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ConditionalBuilder;
