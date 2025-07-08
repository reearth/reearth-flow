import { 
  ariaDescribedByIds, 
  BaseInputTemplateProps, 
  examplesId, 
  FormContextType, 
  RJSFSchema, 
  StrictRJSFSchema 
} from "@rjsf/utils";
import { ChangeEvent, useRef } from "react";

import { TextArea } from "@flow/components";

type DefaultTextAreaProps<T, S extends StrictRJSFSchema, F extends FormContextType> = {
  props: BaseInputTemplateProps<T, S, F>;
  inputProps: any;
  textFieldProps: any;
};

export const DefaultTextArea = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  props,
  inputProps,
  textFieldProps,
}: DefaultTextAreaProps<T, S, F>) => {
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

  const textareaRef = useRef<HTMLTextAreaElement>(null);

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
        disabled={readonly || disabled}
        {...inputProps}
        value={value || value === 0 ? value : ""}
        onChange={handleOnChange}
        {...textFieldProps}
        aria-describedby={ariaDescribedByIds<T>(id, !!schema.examples)}
        aria-required={required}
        aria-invalid={rawErrors.length > 0}
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