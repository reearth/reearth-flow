import { GitBranchIcon, TrashIcon, PlusIcon } from "@phosphor-icons/react";
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
  | "in"
  | "is_empty"
  | "is_not_empty";

type Branch = {
  left: string;
  operator: ConditionOperator;
  right: string;
  result: string;
};

type Props = {
  onExpressionChange: (expression: string) => void;
};

const ConditionalBuilder: React.FC<Props> = ({ onExpressionChange }) => {
  const t = useT();

  const [branches, setBranches] = useState<Branch[]>([
    { left: "", operator: "==", right: "", result: "" },
  ]);
  const [elseResult, setElseResult] = useState("");

  const operators = [
    { value: "==" as const, label: "equals (==)" },
    { value: "!=" as const, label: "not equals (!=)" },
    { value: ">" as const, label: "greater than (>)" },
    { value: "<" as const, label: "less than (<)" },
    { value: ">=" as const, label: "greater or equal (>=)" },
    { value: "<=" as const, label: "less or equal (<=)" },
    { value: "in" as const, label: "in (membership)" },
    { value: "is_empty" as const, label: "is empty" },
    { value: "is_not_empty" as const, label: "is not empty" },
  ];

  const [currentExpression, setCurrentExpression] = useState("");

  const buildConditionStr = (branch: Branch): string => {
    switch (branch.operator) {
      case "is_empty":
        return `${branch.left}.len() == 0`;
      case "is_not_empty":
        return `${branch.left}.len() != 0`;
      case "in":
        return `${branch.left} in ${branch.right || "[]"}`;
      default:
        return `${branch.left} ${branch.operator} ${branch.right}`;
    }
  };

  useEffect(() => {
    if (!branches[0]?.left) {
      setCurrentExpression("");
      return;
    }

    const parts = branches.map((branch, i) => {
      const condStr = buildConditionStr(branch);
      const keyword = i === 0 ? "if" : "else if";
      return `${keyword} ${condStr} { ${branch.result} }`;
    });

    const elseClause = elseResult ? ` else { ${elseResult} }` : "";
    setCurrentExpression(parts.join(" ") + elseClause);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [branches, elseResult]);

  const handleInsertExpression = useCallback(() => {
    if (currentExpression.trim()) {
      onExpressionChange(currentExpression);
    }
  }, [currentExpression, onExpressionChange]);

  const addBranch = useCallback(() => {
    setBranches((prev) => [
      ...prev,
      { left: "", operator: "==", right: "", result: "" },
    ]);
  }, []);

  const removeBranch = useCallback(
    (index: number) => {
      setBranches(branches.filter((_, i) => i !== index));
    },
    [branches],
  );

  const updateBranch = useCallback(
    (index: number, field: keyof Branch, value: string) => {
      const next = [...branches];
      next[index] = { ...next[index], [field]: value };
      setBranches(next);
    },
    [branches],
  );

  const noRightValue = (op: ConditionOperator) =>
    op === "is_empty" || op === "is_not_empty";

  return (
    <div className="flex size-full flex-col gap-4 p-4">
      <div className="flex-shrink-0">
        <h4 className="text-sm font-medium">{t("Conditional Logic")}</h4>
        <p className="text-xs text-muted-foreground">
          {t("Build if / else if / else expressions")}
        </p>
      </div>

      <div className="space-y-4">
        {branches.map((branch, index) => (
          <div key={index} className="rounded border p-4">
            <div className="mb-3 flex items-center justify-between">
              <span className="text-sm font-medium">
                {index === 0 ? t("If") : t("Else If")}
              </span>
              {branches.length > 1 && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => removeBranch(index)}
                  className="h-6 w-6 p-0 text-destructive hover:text-destructive">
                  <TrashIcon className="h-3 w-3" />
                </Button>
              )}
            </div>

            <div className="grid grid-cols-1 gap-3 lg:grid-cols-3">
              <div>
                <Label className="text-xs">{t("Left Value")}</Label>
                <ExpressionInput
                  placeholder='value("type")'
                  value={branch.left}
                  onChange={(v) => updateBranch(index, "left", v)}
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
                  value={branch.operator}
                  onValueChange={(v) =>
                    updateBranch(index, "operator", v as ConditionOperator)
                  }>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {operators.map((op) => (
                      <SelectItem key={op.value} value={op.value}>
                        <div className="text-xs">{op.label}</div>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label className="text-xs">
                  {noRightValue(branch.operator)
                    ? t("(not used)")
                    : branch.operator === "in"
                      ? t("Array / String")
                      : t("Right Value")}
                </Label>
                <ExpressionInput
                  placeholder={
                    branch.operator === "in" ? '["bldg", "tran"]' : '"value"'
                  }
                  value={branch.right}
                  onChange={(v) => updateBranch(index, "right", v)}
                  className="text-sm"
                  label={t("Right Value")}
                  disabled={noRightValue(branch.operator)}
                  allowedExpressionTypes={[
                    "feature-attribute",
                    "environment-variable",
                    "math",
                  ]}
                />
              </div>
            </div>

            <div className="mt-3">
              <Label className="text-xs">{t("Result")}</Label>
              <ExpressionInput
                placeholder='"result"'
                value={branch.result}
                onChange={(v) => updateBranch(index, "result", v)}
                className="text-sm"
                label={t("Result")}
                allowedExpressionTypes={[
                  "file-path",
                  "environment-variable",
                  "feature-attribute",
                  "math",
                ]}
              />
            </div>
          </div>
        ))}

        <Button
          variant="outline"
          size="sm"
          onClick={addBranch}
          className="flex w-full items-center gap-1">
          <PlusIcon className="h-3 w-3" />
          {t("Add Else If")}
        </Button>

        <div className="rounded border p-4">
          <div className="mb-2 flex items-center gap-2">
            <GitBranchIcon className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm font-medium">{t("Else")}</span>
          </div>
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
              "math",
            ]}
          />
        </div>

        {currentExpression && (
          <div className="mt-6 border-t pt-4">
            <div className="mb-3">
              <Label className="text-xs text-muted-foreground">
                {t("Preview")}
              </Label>
              <div className="mt-1 rounded border bg-muted/30 p-2 font-mono text-sm break-all">
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

export default ConditionalBuilder;
