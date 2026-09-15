import Quill from "quill";
import "quill/dist/quill.snow.css";
import { useEffect, useLayoutEffect, useRef } from "react";

import type { WysiwygField as WysiwygFieldNode } from "@flow/lib/schemaForm";

import { FieldRow } from "./FieldRow";
import type { FieldProps } from "./types";
import { useField } from "./useField";

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

const serialize = (quill: Quill): string | undefined => {
  const html = quill.getSemanticHTML();
  return !html || html === QUILL_EMPTY_HTML ? undefined : html;
};

const WysiwygField: React.FC<FieldProps<WysiwygFieldNode>> = ({
  node,
  path,
  value,
  required,
  onChange,
}) => {
  const field = useField(path);

  const containerRef = useRef<HTMLDivElement>(null);
  const quillRef = useRef<Quill | null>(null);
  const isFocusedRef = useRef(false);

  // Quill is created once and then driven imperatively, so the handlers it
  // closes over are kept in refs rather than re-binding the editor.
  const onChangeRef = useRef(onChange);
  const pathRef = useRef(path);
  const fieldRef = useRef(field);
  useLayoutEffect(() => {
    onChangeRef.current = onChange;
    pathRef.current = path;
    fieldRef.current = field;
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

    if (typeof value === "string" && value) {
      quill.setContents(
        quill.clipboard.convert({ html: value }),
        Quill.sources.SILENT,
      );
    }

    quill.on(Quill.events.TEXT_CHANGE, () => {
      onChangeRef.current(pathRef.current, serialize(quill));
    });
    quill.root.addEventListener("focus", () => {
      isFocusedRef.current = true;
      fieldRef.current.onFocus();
    });
    quill.root.addEventListener("blur", () => {
      isFocusedRef.current = false;
      fieldRef.current.onBlur();
    });

    return () => {
      quillRef.current = null;
      container.innerHTML = "";
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    quillRef.current?.enable(!field.readonly);
  }, [field.readonly]);

  useEffect(() => {
    const quill = quillRef.current;
    if (!quill || isFocusedRef.current) return;
    const incoming = typeof value === "string" ? value : "";
    if (quill.getSemanticHTML() !== incoming) {
      quill.setContents(
        quill.clipboard.convert({ html: incoming }),
        Quill.sources.SILENT,
      );
    }
  }, [value]);

  return (
    <FieldRow
      id={field.id}
      label={node.title ?? node.name}
      required={required}
      errors={field.errors}>
      <div
        style={field.awarenessStyle}
        aria-required={required}
        className="w-full overflow-hidden rounded-md border shadow-sm [&_.ql-code-block-container_.ql-code-block]:max-w-full [&_.ql-container]:border-none [&_.ql-container]:bg-transparent [&_.ql-container]:text-sm [&_.ql-editor]:min-h-20 [&_.ql-editor]:text-foreground [&_.ql-editor_p]:max-w-full [&_.ql-editor:focus]:outline-none [&_.ql-toolbar]:border-0! [&_.ql-toolbar]:border-b! [&_.ql-toolbar]:border-border [&_.ql-toolbar]:bg-transparent [&_.ql-toolbar]:p-1 [&_.ql-tooltip]:left-1/2! [&_.ql-tooltip]:-translate-x-1/2!">
        <div ref={containerRef} />
      </div>
    </FieldRow>
  );
};

export { WysiwygField };
