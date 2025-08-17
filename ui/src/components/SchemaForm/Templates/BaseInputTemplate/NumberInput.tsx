import {
  BaseInputTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";
import { useCallback, useRef } from "react";

import { Input } from "@flow/components";

import ActionArea from "../../components/ActionArea";

const NumberInput = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: BaseInputTemplateProps<T, S, F> & {
    onEditorOpen?: () => void;
  },
) => {
  const {
    id,
    placeholder,
    autofocus,
    required,
    readonly,
    disabled,
    value,
    onChange,
    onBlur,
    onFocus,
    onEditorOpen,
    options,
    schema,
    rawErrors = [],
  } = props;
  const defaultValue = useRef(value || schema.default || "");

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const rawValue = e.target.value;
      if (rawValue === "") {
        onChange(options.emptyValue);
        return;
      }

      const isInteger = schema.type === "integer";
      const parsed = isInteger ? parseInt(rawValue, 10) : parseFloat(rawValue);

      if (!isNaN(parsed)) {
        onChange(parsed);
      }
    },
    [onChange, options.emptyValue, schema.type],
  );

  const handleBlur = useCallback(
    (e: React.FocusEvent<HTMLInputElement>) => {
      onBlur?.(id, e.target.value);
    },
    [onBlur, id],
  );

  const handleFocus = useCallback(
    (e: React.FocusEvent<HTMLInputElement>) => {
      onFocus?.(id, e.target.value);
    },
    [onFocus, id],
  );

  const handleReset = useCallback(() => {
    onChange(defaultValue.current);
  }, [onChange]);

  const step = schema.multipleOf || (schema.type === "integer" ? 1 : "any");

  return (
    <div className="flex flex-1 flex-col gap-2">
      <div className="flex items-center gap-2">
        <Input
          id={id}
          name={id}
          type="number"
          placeholder={placeholder}
          autoFocus={autofocus}
          required={required}
          disabled={readonly || disabled}
          value={value || value === 0 ? value : ""}
          onChange={handleChange}
          onBlur={handleBlur}
          onFocus={handleFocus}
          min={schema.minimum}
          max={schema.maximum}
          step={step}
          aria-required={required}
          aria-invalid={rawErrors.length > 0}
          aria-describedby={rawErrors.length > 0 ? `${id}-error` : undefined}
          className={rawErrors.length > 0 ? "border-destructive" : ""}
        />
        <ActionArea
          value={value}
          defaultValue={defaultValue}
          onEditorOpen={onEditorOpen}
          onReset={handleReset}
        />
      </div>
    </div>
  );
};

export { NumberInput };
