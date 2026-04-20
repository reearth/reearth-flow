import {
  ariaDescribedByIds,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  WidgetProps,
} from "@rjsf/utils";
import Quill from "quill";
import "quill/dist/quill.snow.css";
import { useEffect, useLayoutEffect, useRef } from "react";

import { paramsAwarenessStyles } from "../utils/awarenessTemplateStyles";

type CustomWidgetProps<
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
> = WidgetProps<T, S, F> & {
  options: any;
};

const TOOLBAR_OPTIONS = [
  [
    "bold",
    "italic",
    "underline",
    "strike",
    { list: "ordered" },
    { list: "bullet" },
    { color: [] },
    { background: [] },
    "code-block",
    "link",
    "clean",
  ],
];

const QUILL_EMPTY_HTML = "<p><br></p>";

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

  const containerRef = useRef<HTMLDivElement>(null);
  const quillRef = useRef<Quill | null>(null);
  const isFocusedRef = useRef(false);

  const onChangeRef = useRef(onChange);
  const onBlurRef = useRef(onBlur);
  const onFocusRef = useRef(onFocus);
  const onFieldFocusRef = useRef(onFieldFocus);
  const optionsRef = useRef(options);

  useLayoutEffect(() => {
    onChangeRef.current = onChange;
    onBlurRef.current = onBlur;
    onFocusRef.current = onFocus;
    onFieldFocusRef.current = onFieldFocus;
    optionsRef.current = options;
  });

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const editorContainer = container.appendChild(
      container.ownerDocument.createElement("div"),
    );

    const quill = new Quill(editorContainer, {
      theme: "snow",
      modules: { toolbar: TOOLBAR_OPTIONS },
    });

    quillRef.current = quill;

    if (value) {
      const delta = quill.clipboard.convert({ html: value });
      quill.setContents(delta, Quill.sources.SILENT);
    }

    quill.on(Quill.events.TEXT_CHANGE, () => {
      const html = quill.getSemanticHTML();
      onChangeRef.current(
        !html || html === QUILL_EMPTY_HTML
          ? optionsRef.current?.emptyValue
          : html,
      );
    });

    quill.root.addEventListener("focus", () => {
      isFocusedRef.current = true;
      onFocusRef.current(id, quill.root.innerHTML);
      onFieldFocusRef.current?.(id);
    });

    quill.root.addEventListener("blur", () => {
      isFocusedRef.current = false;
      onBlurRef.current(id, quill.root.innerHTML);
      onFieldFocusRef.current?.(null);
    });

    return () => {
      quillRef.current = null;
      container.innerHTML = "";
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id]);

  useEffect(() => {
    quillRef.current?.enable(!(disabled || readonly));
  }, [disabled, readonly]);

  useEffect(() => {
    const quill = quillRef.current;
    if (quill && !isFocusedRef.current) {
      const incoming = value ?? "";
      if (quill.getSemanticHTML() !== incoming) {
        const delta = quill.clipboard.convert({ html: incoming });
        quill.setContents(delta, Quill.sources.SILENT);
      }
    }
  }, [value]);

  return (
    <div
      style={paramsAwarenessStyles(focusedUsers)}
      aria-describedby={ariaDescribedByIds(id)}
      aria-required={required}
      className="w-full overflow-hidden rounded-md border shadow-sm [&_.ql-container]:border-none [&_.ql-container]:bg-transparent [&_.ql-container]:text-sm [&_.ql-editor]:min-h-20 [&_.ql-editor]:text-foreground [&_.ql-editor:focus]:outline-none [&_.ql-toolbar]:border-b [&_.ql-toolbar]:border-border [&_.ql-toolbar]:bg-transparent [&_.ql-toolbar]:p-1 [&_.ql-tooltip]:left-1/2! [&_.ql-tooltip]:-translate-x-1/2!">
      <div ref={containerRef} />
    </div>
  );
};

export { WysiwygWidget };
