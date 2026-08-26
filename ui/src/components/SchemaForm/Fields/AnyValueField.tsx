import { CaretDownIcon } from "@phosphor-icons/react";
import {
  FieldPathId,
  FieldProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";
import { useCallback, useEffect, useState } from "react";

import {
  Checkbox,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Input,
  TextArea,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { paramsAwarenessStyles } from "../utils/awarenessTemplateStyles";

// RJSF v6 uses fieldPathId instead of idSchema
type V6FieldProps<
  T,
  S extends StrictRJSFSchema,
  F extends FormContextType,
> = Omit<FieldProps<T, S, F>, "idSchema"> & { fieldPathId: FieldPathId };

type ValueType = "string" | "number" | "boolean" | "null" | "json";

const VALUE_TYPES: ValueType[] = [
  "string",
  "number",
  "boolean",
  "null",
  "json",
];

export const detectValueType = (value: unknown): ValueType => {
  if (value === null) return "null";
  if (typeof value === "string") return "string";
  if (typeof value === "number") return "number";
  if (typeof value === "boolean") return "boolean";
  if (typeof value === "object") return "json";
  return "string";
};

// Converts the current value to the newly picked type so switching types keeps
// as much of what the user already typed as it reasonably can.
export const castToType = (value: unknown, type: ValueType): unknown => {
  switch (type) {
    case "string":
      if (value === null || value === undefined) return "";
      return typeof value === "object" ? JSON.stringify(value) : String(value);
    case "number": {
      const asNumber = Number(value);
      return Number.isFinite(asNumber) ? asNumber : 0;
    }
    case "boolean":
      return Boolean(value);
    case "null":
      return null;
    case "json":
      return typeof value === "object" && value !== null ? value : {};
  }
};

const stringifyJson = (value: unknown): string => {
  if (typeof value === "object" && value !== null) {
    return JSON.stringify(value, null, 2);
  }
  return "{}";
};

/**
 * Renders schema nodes that declare no `type`. The engine emits these for
 * parameters typed as a raw JSON value in Rust (`serde_json::Value`) — for
 * example Attribute Range Mapper's `outputValue` or Null Attribute Mapper's
 * `replacement` — which RJSF would otherwise render as an "unsupported field"
 * error. Registered as RJSF's `FallbackField`, so it also catches any other
 * schema RJSF cannot map to a field.
 *
 * The label comes from `FieldTemplate`; this renders the controls only.
 */
const AnyValueField = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  fieldPathId,
  formData,
  onChange,
  onBlur,
  onFocus,
  readonly,
  disabled,
  schema,
  registry,
}: V6FieldProps<T, S, F>) => {
  const t = useT();
  const formContext = registry?.formContext as
    | {
        fieldFocusMap?: Record<string, any[]>;
        onFieldFocus?: (fieldId: string | null) => void;
      }
    | undefined;
  const { fieldFocusMap, onFieldFocus } = formContext ?? {};

  const id = fieldPathId.$id;
  const focusedUsers = fieldFocusMap?.[id] ?? [];
  const awarenessStyle = paramsAwarenessStyles(focusedUsers);
  const isDisabled = Boolean(readonly || disabled);

  // The type shown is derived from the value itself wherever the value has one,
  // so a value arriving from outside the field — a collaborator's edit, a form
  // reset — brings its own control with it. `pickedType` only fills in while
  // there is no value to read a type from.
  const detectedType =
    formData === undefined ? undefined : detectValueType(formData);
  const [pickedType, setPickedType] = useState<ValueType>(
    detectedType ?? "string",
  );
  const valueType = detectedType ?? pickedType;

  // The JSON textarea keeps its own text so a half-typed object is not thrown
  // away on every keystroke; only parseable text is pushed to the form.
  const [jsonText, setJsonText] = useState<string>(() =>
    stringifyJson(formData),
  );
  const [jsonError, setJsonError] = useState<string | null>(null);

  useEffect(() => {
    if (valueType !== "json") return;
    let parsed: unknown;
    try {
      parsed = JSON.parse(jsonText);
    } catch {
      return; // Mid-edit text: leave it alone.
    }
    // Equal means the value came from this textarea; anything else replaced it.
    if (JSON.stringify(parsed) === JSON.stringify(formData)) return;
    setJsonText(stringifyJson(formData));
    setJsonError(null);
  }, [formData, valueType, jsonText]);

  const typeLabels: Record<ValueType, string> = {
    string: t("Text"),
    number: t("Number"),
    boolean: t("True / False"),
    null: t("Null"),
    json: t("JSON"),
  };

  const emit = useCallback(
    (value: unknown) => {
      // fieldPathId.path is the full path from root — required by RJSF v6 so it
      // sets only this field's slice of formData, not the entire form root.
      onChange(value as T, fieldPathId.path, undefined, id);
    },
    [onChange, fieldPathId.path, id],
  );

  const handleFocus = useCallback(() => {
    onFocus?.(id, formData);
    onFieldFocus?.(id);
  }, [onFocus, onFieldFocus, id, formData]);

  const handleBlur = useCallback(() => {
    onBlur?.(id, formData);
    onFieldFocus?.(null);
  }, [onBlur, onFieldFocus, id, formData]);

  const handleTypeChange = useCallback(
    (nextType: ValueType) => {
      setPickedType(nextType);
      const nextValue = castToType(formData, nextType);
      if (nextType === "json") {
        setJsonText(stringifyJson(nextValue));
        setJsonError(null);
      }
      emit(nextValue);
    },
    [formData, emit],
  );

  const handleJsonChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      const text = e.target.value;
      setJsonText(text);
      try {
        emit(JSON.parse(text));
        setJsonError(null);
      } catch {
        setJsonError(t("Invalid JSON"));
      }
    },
    [emit, t],
  );

  return (
    <div className="flex min-w-0 flex-1 flex-col gap-2">
      <div className="flex min-w-0 items-center gap-2">
        <DropdownMenu modal={true}>
          <DropdownMenuTrigger
            className="flex h-8 shrink-0 items-center justify-between gap-2 rounded border bg-background px-3 hover:bg-accent"
            disabled={isDisabled}
            aria-label={t("Value type")}>
            <span>{typeLabels[valueType]}</span>
            <CaretDownIcon className="size-4" />
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            {VALUE_TYPES.map((type) => (
              <DropdownMenuItem
                key={type}
                onClick={() => handleTypeChange(type)}
                className={valueType === type ? "bg-accent" : ""}>
                {typeLabels[type]}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>

        {valueType === "string" && (
          <Input
            id={id}
            className="min-w-0 flex-1"
            style={awarenessStyle}
            value={typeof formData === "string" ? formData : ""}
            placeholder={schema.description ?? t("Enter value...")}
            readOnly={readonly}
            disabled={disabled}
            onChange={(e) => emit(e.target.value)}
            onFocus={handleFocus}
            onBlur={handleBlur}
          />
        )}

        {valueType === "number" && (
          <Input
            id={id}
            type="number"
            className="min-w-0 flex-1"
            style={awarenessStyle}
            value={typeof formData === "number" ? String(formData) : ""}
            readOnly={readonly}
            disabled={disabled}
            onChange={(e) =>
              emit(e.target.value === "" ? undefined : Number(e.target.value))
            }
            onFocus={handleFocus}
            onBlur={handleBlur}
          />
        )}

        {valueType === "boolean" && (
          <Checkbox
            id={id}
            style={awarenessStyle}
            checked={formData === true}
            disabled={isDisabled}
            onCheckedChange={(checked) => emit(Boolean(checked))}
            onFocus={handleFocus}
            onBlur={handleBlur}
          />
        )}

        {valueType === "null" && (
          <p className="flex-1 text-xs text-muted-foreground">
            {t("No value is written for this entry.")}
          </p>
        )}
      </div>

      {valueType === "json" && (
        <>
          <TextArea
            id={id}
            className="font-mono text-xs"
            style={awarenessStyle}
            value={jsonText}
            readOnly={readonly}
            disabled={disabled}
            onChange={handleJsonChange}
            onFocus={handleFocus}
            onBlur={handleBlur}
          />
          {jsonError && (
            <p className="text-xs text-destructive" role="alert">
              {jsonError}
            </p>
          )}
        </>
      )}
    </div>
  );
};

export { AnyValueField };
