import {
  ariaDescribedByIds,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
import { useCallback, useRef } from "react";
import {
  ContentEditableEvent,
  Editor as WysiwygEditor,
  EditorProvider as WysiwygEditorProvider,
} from "react-simple-wysiwyg";

import { paramsAwarenessStyles } from "../utils/awarenessTemplateStyles";

import { WysiwygToolbar } from "./components";

type CustomWidgetProps<
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
> = WidgetProps<T, S, F> & {
  options: any;
};

const WysiwygWidget = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  id,
  value,
  required,
  disabled,
  registry,
  readonly,
  onBlur,
  onFocus,
  onChange,
  options,
}: CustomWidgetProps<T, S, F>) => {
  const formContext = registry?.formContext;
  const { fieldFocusMap, onFieldFocus } = formContext ?? {};
  const focusedUsers = fieldFocusMap?.[id] ?? [];

  const isFocusedRef = useRef(false);

  const handleChange = useCallback(
    (e: ContentEditableEvent) => {
      const html = e.target.value;
      onChange(html === "" ? options?.emptyValue : html);
    },
    [onChange, options?.emptyValue],
  );

  const handleFocus = useCallback(() => {
    isFocusedRef.current = true;
    onFocus(id, value);
    onFieldFocus?.(id);
  }, [onFocus, id, onFieldFocus, value]);

  const handleBlur = useCallback(() => {
    isFocusedRef.current = false;
    onBlur(id, value);
    onFieldFocus?.(null);
  }, [onBlur, id, onFieldFocus, value]);

  return (
    <div
      style={paramsAwarenessStyles(focusedUsers)}
      className="w-full rounded-md border shadow-sm [&_.rsw-btn]:rounded [&_.rsw-btn]:p-1 [&_.rsw-btn]:text-primary [&_.rsw-btn:hover]:bg-accent [&_.rsw-ce]:min-h-20 [&_.rsw-ce]:px-3 [&_.rsw-ce]:py-2 [&_.rsw-ce]:text-sm [&_.rsw-ce:focus]:outline-none [&_.rsw-editor]:border-none [&_.rsw-editor]:bg-transparent [&_.rsw-toolbar]:flex [&_.rsw-toolbar]:gap-1 [&_.rsw-toolbar]:border-b [&_.rsw-toolbar]:border-border [&_.rsw-toolbar]:bg-transparent [&_.rsw-toolbar]:p-1">
      <WysiwygEditorProvider>
        <WysiwygToolbar />
        <WysiwygEditor
          id={id}
          disabled={disabled || readonly}
          className="border"
          value={value}
          onChange={handleChange}
          onBlur={handleBlur}
          onFocus={handleFocus}
          aria-describedby={ariaDescribedByIds(id)}
          aria-required={required}
        />
      </WysiwygEditorProvider>
    </div>
  );
};

export { WysiwygWidget };
