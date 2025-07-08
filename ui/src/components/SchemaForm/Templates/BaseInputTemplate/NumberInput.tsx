import {
  BaseInputTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";
import { ChangeEvent, useCallback } from "react";

import { Button, Input } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type NumberInputProps<
  T,
  S extends StrictRJSFSchema,
  F extends FormContextType,
> = {
  props: BaseInputTemplateProps<T, S, F>;
  inputProps: any;
};

export const NumberInput = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  props,
  inputProps,
}: NumberInputProps<T, S, F>) => {
  const {
    id,
    placeholder,
    autofocus,
    required,
    readonly,
    disabled,
    value,
    onChange,
    onChangeOverride,
    options,
    schema,
    rawErrors = [],
  } = props;

  const t = useT();
  const defaultValue = schema.default ?? "";
  const { step, min, max } = inputProps.inputProps || {};

  const handleChange = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      const inputValue = e.target.value;

      if (inputValue === "") {
        return onChangeOverride || onChange(options.emptyValue);
      }

      const numericValue = parseFloat(inputValue);
      if (!isNaN(numericValue)) {
        return onChangeOverride || onChange(numericValue);
      }

      // If parsing fails, don't update (maintains current value)
    },
    [onChangeOverride, onChange, options.emptyValue],
  );

  const displayValue =
    value !== null && value !== undefined ? String(value) : "";

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <Input
          id={id}
          name={id}
          type="number"
          placeholder={placeholder}
          autoFocus={autofocus}
          required={required}
          disabled={readonly || disabled}
          value={displayValue}
          onChange={handleChange}
          step={step}
          min={min}
          max={max}
          aria-required={required}
          aria-invalid={rawErrors.length > 0}
          aria-describedby={rawErrors.length > 0 ? `${id}-error` : undefined}
        />
        {value !== defaultValue && defaultValue !== "" && (
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => onChange(defaultValue)}
            disabled={readonly || disabled}
            className="h-9 px-2"
            aria-label={`Reset value to default: ${defaultValue}`}>
            {t("Reset")}
          </Button>
        )}
      </div>
      {rawErrors.length > 0 &&
        rawErrors.map((error, i) => (
          <p key={i} className="text-xs text-destructive" id={`${id}-error`}>
            {error}
          </p>
        ))}
    </div>
  );
};
