import {
  BaseInputTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import { Button, Input } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type TextInputProps<
  T,
  S extends StrictRJSFSchema,
  F extends FormContextType,
> = {
  props: BaseInputTemplateProps<T, S, F>;
  inputProps: any;
};

export const TextInput = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  props,
  inputProps,
}: TextInputProps<T, S, F>) => {
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
  const defaultValue = schema.default || "";

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
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
          aria-required={required}
        />
        {value !== defaultValue && (
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => onChange(defaultValue)}
            disabled={readonly || disabled}
            className="h-9 px-2"
            aria-label={`Reset value to default: ${defaultValue}`}>
            {t("Reset Value")}
          </Button>
        )}
      </div>
    </div>
  );
};
