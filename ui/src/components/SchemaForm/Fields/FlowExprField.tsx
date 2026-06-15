import { PencilLineIcon, ArrowUDownLeftIcon } from "@phosphor-icons/react";
import {
  FieldPathId,
  FieldProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";
import { useCallback, useRef } from "react";

import { Input } from "@flow/components";
import { IconButton } from "@flow/components/buttons";
import {
  FieldContext,
  createFieldContext,
} from "@flow/features/Editor/components/ParamsDialog/utils/fieldUtils";
import { useT } from "@flow/lib/i18n";

import { ExtendedFormContext } from "../Templates/BaseInputTemplate";
import { paramsAwarenessStyles } from "../utils/awarenessTemplateStyles";

export type CodeValue = {
  type: "flowExpr" | "string";
  value: string;
};

// RJSF v6 uses fieldPathId instead of idSchema
type V6FieldProps<
  T,
  S extends StrictRJSFSchema,
  F extends FormContextType,
> = Omit<FieldProps<T, S, F>, "idSchema"> & { fieldPathId: FieldPathId };

const FlowExprField = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  fieldPathId,
  name,
  formData,
  onChange,
  readonly,
  disabled,
  required,
  schema,
  registry,
}: V6FieldProps<T, S, F>) => {
  const t = useT();
  const formContext = registry.formContext as ExtendedFormContext;
  const { onFlowExprEditorOpen, onFieldFocus, fieldFocusMap } =
    formContext || {};

  const id = fieldPathId.$id;
  const focusedUsers = fieldFocusMap?.[id] ?? [];
  const awarenessStyle = paramsAwarenessStyles(focusedUsers);

  const codeValue = formData as CodeValue | undefined;
  const isExpression = codeValue?.type === "flowExpr";
  const label = schema.title || name;
  const defaultValue = useRef<CodeValue | undefined>(codeValue);

  const handleInlineChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      // fieldPathId.path is the full path from root — required by RJSF v6 so it
      // sets only this field's slice of formData, not the entire form root.
      onChange(
        { type: "string", value: e.target.value } as any,
        fieldPathId.path,
        undefined,
        id,
      );
    },
    [onChange, fieldPathId.path, id],
  );

  const handleReset = useCallback(() => {
    onChange(
      (defaultValue.current ?? { type: "string", value: "" }) as any,
      fieldPathId.path,
      undefined,
      id,
    );
  }, [onChange, fieldPathId.path, id]);

  const handleEditorOpen = useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      e.preventDefault();
      if (!onFlowExprEditorOpen) return;
      onFieldFocus?.(id);
      const fieldContext: FieldContext = createFieldContext({
        id,
        name,
        value: codeValue ?? { type: "string", value: "" },
        schema,
      });
      onFlowExprEditorOpen(fieldContext);
    },
    [id, name, codeValue, schema, onFlowExprEditorOpen, onFieldFocus],
  );

  const labelNode = (
    <div className="flex flex-row gap-1">
      <p className="shrink-0 font-light">{label}</p>
      {required && <p className="h-2 font-thin text-destructive">*</p>}
    </div>
  );

  if (isExpression) {
    return (
      <div className="flex flex-1 items-center gap-6">
        {labelNode}
        <div className="flex min-w-0 flex-1 items-center gap-2">
          <div
            className="flex min-w-0 flex-1 items-center gap-1 rounded-md border bg-muted/30 px-3 py-2 text-sm"
            style={awarenessStyle}>
            <span className="shrink-0 rounded bg-primary/10 px-1 py-0.5 font-mono text-xs text-primary">
              {t("expr")}
            </span>
            <span className="min-w-0 truncate font-mono text-xs text-muted-foreground">
              {codeValue?.value || (
                <em className="not-italic opacity-50">{t("(empty)")}</em>
              )}
            </span>
          </div>
          <IconButton
            icon={<PencilLineIcon />}
            tooltipText={t("Open FlowExpr Editor")}
            onClick={handleEditorOpen}
            disabled={!onFlowExprEditorOpen || readonly || disabled}
          />
          <IconButton
            icon={<ArrowUDownLeftIcon />}
            tooltipText={t("Reset to Default")}
            onClick={handleReset}
            disabled={readonly || disabled}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-1 items-center gap-6">
      {labelNode}
      <div className="flex min-w-0 flex-1 items-center gap-2">
        <Input
          className="min-w-0 flex-1"
          style={awarenessStyle}
          value={codeValue?.value ?? ""}
          onChange={handleInlineChange}
          onFocus={() => onFieldFocus?.(id)}
          onBlur={() => onFieldFocus?.(null)}
          placeholder={schema.description || t("Enter value...")}
          readOnly={readonly}
          disabled={disabled}
        />
        <IconButton
          icon={<PencilLineIcon />}
          tooltipText={t("Open FlowExpr Editor")}
          onClick={handleEditorOpen}
          disabled={!onFlowExprEditorOpen || readonly || disabled}
        />
        <IconButton
          icon={<ArrowUDownLeftIcon />}
          tooltipText={t("Reset to Default")}
          onClick={handleReset}
          disabled={
            JSON.stringify(codeValue) ===
              JSON.stringify(defaultValue.current) ||
            readonly ||
            disabled
          }
        />
      </div>
    </div>
  );
};

export { FlowExprField };
