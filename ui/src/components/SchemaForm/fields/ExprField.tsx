import { ArrowUDownLeftIcon, PencilLineIcon } from "@phosphor-icons/react";
import { useCallback, useRef } from "react";

import { Input } from "@flow/components";
import { IconButton } from "@flow/components/buttons";
import { useT } from "@flow/lib/i18n";
import type { ExprField as ExprFieldNode } from "@flow/lib/schemaForm";

import { FieldRow } from "./FieldRow";
import type { FieldProps } from "./types";
import { useField } from "./useField";

export type CodeValue = {
  type: "flowExpr" | "string";
  value: string;
};

const ExprField: React.FC<FieldProps<ExprFieldNode>> = ({
  node,
  path,
  value,
  required,
  hideLabel,
  onChange,
}) => {
  const t = useT();
  const field = useField(path);

  const codeValue = value as CodeValue | null | undefined;
  const stringAllowed = node.allowedTypes.includes("string");
  const fallbackType: CodeValue["type"] = stringAllowed ? "string" : "flowExpr";
  // A schema that admits only `flowExpr` has no literal form to fall back to,
  // so the field is always the expression chip.
  const isExpression = codeValue?.type === "flowExpr" || !stringAllowed;
  const defaultValue = useRef(codeValue);

  const openEditor =
    node.flavor === "python"
      ? field.onPythonEditorOpen
      : field.onFlowExprEditorOpen;

  const handleInlineChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      onChange(path, { type: "string", value: event.target.value });
    },
    [onChange, path],
  );

  const handleReset = useCallback(() => {
    onChange(path, defaultValue.current ?? { type: fallbackType, value: "" });
  }, [onChange, path, fallbackType]);

  const handleEditorOpen = useCallback(
    (event: React.MouseEvent<HTMLButtonElement>) => {
      event.preventDefault();
      if (!openEditor) return;
      field.onFocus();
      openEditor({
        key: field.key,
        name: node.name,
        path,
        value: codeValue ?? { type: fallbackType, value: "" },
        schema: node.raw,
        fieldName: node.name,
      });
    },
    [openEditor, field, node, path, codeValue, fallbackType],
  );

  const editorLabel =
    node.flavor === "python"
      ? t("Open Python Editor")
      : t("Open FlowExpr Editor");

  const buttons = (
    <>
      {/* The tooltip is a hover affordance only, so the label is stated too. */}
      <IconButton
        icon={<PencilLineIcon />}
        tooltipText={editorLabel}
        aria-label={editorLabel}
        onClick={handleEditorOpen}
        disabled={!openEditor || field.readonly}
      />
      <IconButton
        icon={<ArrowUDownLeftIcon />}
        tooltipText={t("Reset to Default")}
        aria-label={t("Reset to Default")}
        onClick={handleReset}
        disabled={field.readonly}
      />
    </>
  );

  return (
    <FieldRow
      id={field.id}
      label={node.title ?? node.name}
      required={required}
      errors={field.errors}
      hideLabel={hideLabel}>
      <div className="flex min-w-0 flex-1 items-center gap-2">
        {isExpression ? (
          <div
            className={`flex min-w-0 flex-1 items-center gap-1 rounded-md border bg-muted/30 px-3 py-2 text-sm ${
              field.hasErrors ? "border-destructive" : ""
            }`}
            style={field.awarenessStyle}>
            <span className="shrink-0 rounded bg-primary/10 px-1 py-0.5 font-mono text-xs text-primary">
              {t("expr")}
            </span>
            <span className="min-w-0 truncate font-mono text-xs text-muted-foreground">
              {codeValue?.value || (
                <em className="not-italic opacity-50">{t("(empty)")}</em>
              )}
            </span>
          </div>
        ) : (
          <Input
            id={field.id}
            className={`min-w-0 flex-1 ${field.hasErrors ? "border-destructive" : ""}`}
            style={field.awarenessStyle}
            value={codeValue?.value ?? ""}
            onChange={handleInlineChange}
            onFocus={field.onFocus}
            onBlur={field.onBlur}
            placeholder={node.description || t("Enter value...")}
            disabled={field.readonly}
            aria-invalid={field.hasErrors}
            aria-describedby={field.describedBy}
          />
        )}
        {buttons}
      </div>
    </FieldRow>
  );
};

export { ExprField };
