import {
  BaseInputTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";
import { useCallback, useRef } from "react";

import { Input } from "@flow/components";

import ActionArea from "../../components/ActionArea";

const ColorInput = <
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
  const defaultValue = useRef(value || schema.default || "#000000");

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = e.target.value;
      onChange(newValue === "" ? options.emptyValue : newValue);
    },
    [onChange, options.emptyValue],
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

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <Input
          id={id}
          name={id}
          type="color"
          placeholder={placeholder}
          autoFocus={autofocus}
          required={required}
          disabled={readonly || disabled}
          value={value || defaultValue.current}
          onChange={handleChange}
          onBlur={handleBlur}
          onFocus={handleFocus}
          aria-required={required}
          aria-invalid={rawErrors.length > 0}
          aria-describedby={rawErrors.length > 0 ? `${id}-error` : undefined}
          className={`${rawErrors.length > 0 ? "border-destructive" : ""} h-7 w-12 p-0`}
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

export { ColorInput };
