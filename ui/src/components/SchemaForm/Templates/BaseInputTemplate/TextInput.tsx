import { ArrowUDownLeftIcon, PencilLineIcon } from "@phosphor-icons/react";
import {
  BaseInputTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";
import { useCallback, useRef } from "react";

import { IconButton, Input } from "@flow/components";
import { useT } from "@flow/lib/i18n";

const TextInput = <
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
  const t = useT();
  const defaultValue = useRef(value || schema.default || "");

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

  const handleEditorOpen = useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      e.preventDefault();
      onEditorOpen?.();
    },
    [onEditorOpen],
  );

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <Input
          id={id}
          name={id}
          type="text"
          placeholder={placeholder}
          autoFocus={autofocus}
          required={required}
          disabled={readonly || disabled}
          value={value || value === 0 ? value : ""}
          onChange={handleChange}
          onBlur={handleBlur}
          onFocus={handleFocus}
          aria-required={required}
          aria-invalid={rawErrors.length > 0}
          aria-describedby={rawErrors.length > 0 ? `${id}-error` : undefined}
          className={rawErrors.length > 0 ? "border-destructive" : ""}
        />
        <IconButton
          icon={<PencilLineIcon />}
          tooltipText={t("Open Editor")}
          onClick={handleEditorOpen}
          disabled={!onEditorOpen}
        />
        <IconButton
          icon={<ArrowUDownLeftIcon />}
          disabled={value === defaultValue.current}
          tooltipText={t("Reset to Default")}
          aria-label={`Reset value to default: ${defaultValue.current}`}
          onClick={handleReset}
        />
      </div>
    </div>
  );
};

export { TextInput };
