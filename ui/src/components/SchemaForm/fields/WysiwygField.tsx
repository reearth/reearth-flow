import Quill from "quill";
import "quill/dist/quill.snow.css";
import { useCallback, useEffect, useLayoutEffect, useRef } from "react";

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
  // The latest value from props, so a remote edit that arrived while the
  // editor had focus can be applied the moment it loses focus.
  const valueRef = useRef(value);
  valueRef.current = value;

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

  /**
   * Bring the editor in line with the value held in props.
   *
   * Applying a remote edit under the caret would move it mid-sentence, so an
   * incoming value is skipped while the editor has focus — but it then has to
   * be applied on blur. Without that, the skipped edit was never applied at
   * all, and the next local keystroke overwrote the collaborator's text.
   */
  const syncFromProps = useCallback(() => {
    const quill = quillRef.current;
    if (!quill || isFocusedRef.current) return;
    const incoming =
      typeof valueRef.current === "string" ? valueRef.current : "";
    if (quill.getSemanticHTML() === incoming) return;
    quill.setContents(
      quill.clipboard.convert({ html: incoming }),
      Quill.sources.SILENT,
    );
  }, []);
  const syncRef = useRef(syncFromProps);
  syncRef.current = syncFromProps;

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
      syncRef.current();
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
    syncFromProps();
  }, [value, syncFromProps]);

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
