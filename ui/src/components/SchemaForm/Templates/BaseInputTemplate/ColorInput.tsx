import {
  BaseInputTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import { Button, Input } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type ColorInputProps<
  T,
  S extends StrictRJSFSchema,
  F extends FormContextType,
> = {
  props: BaseInputTemplateProps<T, S, F>;
  inputProps: any;
};

export const ColorInput = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  props,
  inputProps,
}: ColorInputProps<T, S, F>) => {
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
  } = props;

  const t = useT();
  const defaultColor = schema.default || "#000000";

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between gap-2">
        <Input
          id={id}
          name={id}
          placeholder={placeholder}
          autoFocus={autofocus}
          required={required}
          disabled={readonly || disabled}
          {...inputProps}
          value={value || value === 0 ? value : ""}
          onChange={(e) =>
            onChangeOverride ||
            onChange(
              e.target.value === "" ? options.emptyValue : e.target.value,
            )
          }
          className="h-9 w-[200px] cursor-pointer p-1"
          aria-label={`Color picker: ${placeholder || "Select a color"}`}
          aria-required={required}
        />
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => onChange(defaultColor)}
          disabled={value === defaultColor}
          className="h-9 px-2"
          aria-label={`Reset color to default: ${defaultColor}`}>
          {t("Reset")}
        </Button>
      </div>
    </div>
  );
};
