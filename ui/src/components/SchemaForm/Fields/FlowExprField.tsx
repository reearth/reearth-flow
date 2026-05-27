import { PencilLineIcon } from "@phosphor-icons/react";
import {
  FieldPathId,
  FieldProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";
import { useCallback } from "react";

import { IconButton } from "@flow/components/buttons";
import {
  FieldContext,
  createFieldContext,
} from "@flow/features/Editor/components/ParamsDialog/utils/fieldUtils";
import { useT } from "@flow/lib/i18n";

import { ExtendedFormContext } from "../Templates/BaseInputTemplate";

export type CodeValue = {
  type: "flowExpr" | "string";
  value: string;
};

// RJSF v6 uses fieldPathId instead of idSchema
type V6FieldProps<T, S extends StrictRJSFSchema, F extends FormContextType> =
  Omit<FieldProps<T, S, F>, "idSchema"> & { fieldPathId: FieldPathId };

const FlowExprField = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  fieldPathId,
  name,
  formData,
  readonly,
  disabled,
  schema,
  registry,
}: V6FieldProps<T, S, F>) => {
  const t = useT();
  const formContext = registry.formContext as ExtendedFormContext;
  const { onFlowExprEditorOpen } = formContext || {};

  const id = fieldPathId.$id;
  const codeValue = formData as CodeValue | undefined;
  const displayValue = codeValue?.value ?? "";
  const modeLabel = codeValue?.type === "string" ? t("literal") : t("expr");

  const handleEditorOpen = useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      e.preventDefault();
      if (!onFlowExprEditorOpen) return;
      const fieldContext: FieldContext = createFieldContext({
        id,
        name,
        value: formData,
        schema,
      });
      onFlowExprEditorOpen(fieldContext);
    },
    [id, name, formData, schema, onFlowExprEditorOpen],
  );

  return (
    <div className="flex w-full items-center gap-2">
      <div className="flex min-w-0 flex-1 items-center gap-1 rounded-md border bg-muted/30 px-3 py-2 text-sm">
        {codeValue ? (
          <>
            <span className="shrink-0 rounded bg-primary/10 px-1 py-0.5 font-mono text-xs text-primary">
              {modeLabel}
            </span>
            <span className="truncate font-mono text-xs text-muted-foreground">
              {displayValue || (
                <em className="not-italic opacity-50">{t("(empty)")}</em>
              )}
            </span>
          </>
        ) : (
          <span className="text-muted-foreground opacity-50">
            {t("No value set")}
          </span>
        )}
      </div>
      <IconButton
        icon={<PencilLineIcon />}
        tooltipText={t("Open FlowExpr Editor")}
        onClick={handleEditorOpen}
        disabled={!onFlowExprEditorOpen || readonly || disabled}
      />
    </div>
  );
};

export { FlowExprField };
