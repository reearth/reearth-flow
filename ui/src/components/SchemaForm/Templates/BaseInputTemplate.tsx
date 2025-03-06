import {
  ariaDescribedByIds,
  BaseInputTemplateProps,
  examplesId,
  getInputProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";
import { ChangeEvent, useRef } from "react";

import { Button, Input, TextArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";

/** The `BaseInputTemplate` is the template to use to render the basic `<input>` component for the `core` theme.
 * It is used as the template for rendering many of the <input> based widgets that differ by `type` and callbacks only.
 * It can be customized/overridden for other themes or individual implementations as needed.
 *
 * @param props - The `WidgetProps` for this template
 */
const BaseInputTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: BaseInputTemplateProps<T, S, F>,
) => {
  const {
    id,
    name, // remove this from textFieldProps
    placeholder,
    required,
    readonly,
    disabled,
    type,
    label,
    hideLabel,
    hideError,
    value,
    onChange,
    onChangeOverride,
    onBlur,
    onFocus,
    autofocus,
    options,
    schema,
    uiSchema,
    rawErrors = [],
    errorSchema,
    formContext,
    registry,
    InputLabelProps,
    ...textFieldProps
  } = props;
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const inputProps = getInputProps<T, S, F>(schema, type, options);
  // Now we need to pull out the step, min, max into an inner `inputProps` for material-ui
  const { step, min, max, ...rest } = inputProps;
  const otherProps = {
    inputProps: {
      step,
      min,
      max,
      ...(schema.examples ? { list: examplesId<T>(id) } : undefined),
    },
    ...rest,
  };
  const handleOnChange = ({
    target: { value },
  }: ChangeEvent<HTMLTextAreaElement>) => {
    const textarea = textareaRef.current;
    if (textarea) {
      textarea.style.height = "auto";
      textarea.style.height = `${textarea.scrollHeight}px`;
    }
    return (
      onChangeOverride || onChange(value === "" ? options.emptyValue : value)
    );
  };
  const t = useT();
  // For most text-based params we want TextArea. But for certain schema format types, we want Input to get the appropriate styling @billcookie
  if (schema.format === "color") {
    const defaultColor = schema.default || "#000000";
    return (
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-2">
          <Input
            id={id}
            name={id}
            placeholder={placeholder}
            autoFocus={autofocus}
            required={required}
            disabled={disabled || readonly}
            {...otherProps}
            value={value || value === 0 ? value : ""}
            onChange={(e) =>
              onChangeOverride ||
              onChange(
                e.target.value === "" ? options.emptyValue : e.target.value,
              )
            }
            className="h-9 w-full cursor-pointer p-1"
          />
          {value !== defaultColor && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => onChange(defaultColor)}
              className="h-9 px-2">
              {t("Reset Color")}
            </Button>
          )}
        </div>
      </div>
    );
  }
  return (
    <>
      <TextArea
        ref={textareaRef}
        id={id}
        name={id}
        rows={1}
        placeholder={placeholder}
        autoFocus={autofocus}
        required={required}
        disabled={disabled || readonly}
        {...otherProps}
        value={value || value === 0 ? value : ""}
        onChange={handleOnChange}
        {...textFieldProps}
        aria-describedby={ariaDescribedByIds<T>(id, !!schema.examples)}
      />
      {Array.isArray(schema.examples) && (
        <datalist id={examplesId<T>(id)}>
          {(schema.examples as string[])
            .concat(
              schema.default && !schema.examples.includes(schema.default)
                ? ([schema.default] as string[])
                : [],
            )
            .map((example: string) => {
              return <option key={example} value={example} />;
            })}
        </datalist>
      )}
      {rawErrors.length > 0 &&
        rawErrors.map((e, i) => (
          <p key={i} className="text-xs text-destructive">
            {e}
          </p>
        ))}
    </>
  );
};

export { BaseInputTemplate };
