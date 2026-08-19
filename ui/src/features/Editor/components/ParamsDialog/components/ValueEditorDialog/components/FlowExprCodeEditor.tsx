import {
  useCallback,
  useRef,
  useEffect,
  useLayoutEffect,
  useState,
  useImperativeHandle,
  forwardRef,
} from "react";

import { TextArea } from "@flow/components";

import {
  getCompletionContext,
  type CompletionContext,
} from "./flowExprAttributeContext";
import FlowExprAutocomplete, {
  type FlowExprAutocompleteRef,
} from "./FlowExprAutocomplete";
import { type AutocompleteSuggestion } from "./flowExprConstants";
import FlowExprSyntaxHighlighter from "./FlowExprSyntaxHighlighter";
import {
  validateFlowExprCode,
  type ValidationError,
} from "./FlowExprValidator";

export type FlowExprCodeEditorRef = {
  insertAtCursor: (text: string) => void;
  focus: () => void;
};

type Props = {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
  attributeSuggestions?: AutocompleteSuggestion[];
  variableSuggestions?: AutocompleteSuggestion[];
  "data-testid"?: string;
  "aria-label"?: string;
  "data-placeholder"?: string;
};

const FlowExprCodeEditor = forwardRef<FlowExprCodeEditorRef, Props>(
  (
    {
      value = "",
      onChange,
      placeholder,
      className = "",
      attributeSuggestions,
      variableSuggestions,
      ...props
    },
    ref,
  ) => {
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const highlightRef = useRef<HTMLDivElement>(null);
    const placeholderRef = useRef<HTMLDivElement>(null);
    const errorOverlayRef = useRef<HTMLDivElement>(null);
    const autocompleteRef = useRef<FlowExprAutocompleteRef>(null);

    // Caret position, mirrored into state so the autocomplete recomputes its
    // context whenever the caret moves — not only when the text changes.
    const [caret, setCaret] = useState(0);
    // The dropdown is armed by editing and disarmed by anything that means
    // "I'm done here": Escape, moving the caret, accepting a suggestion.
    const [autocompleteArmed, setAutocompleteArmed] = useState(false);
    // Applied after the controlled value lands, which otherwise parks the
    // caret at the end of the textarea.
    const pendingCaretRef = useRef<number | null>(null);

    const [validationErrors, setValidationErrors] = useState<ValidationError[]>(
      [],
    );
    const validationTimeoutRef = useRef<NodeJS.Timeout | null>(null);

    const applyCaret = useCallback(
      (textarea: HTMLTextAreaElement, pos: number) => {
        textarea.setSelectionRange(pos, pos);
        textarea.focus();
        setCaret(pos);
      },
      [],
    );

    useLayoutEffect(() => {
      const pos = pendingCaretRef.current;
      if (pos === null || !textareaRef.current) return;
      pendingCaretRef.current = null;
      applyCaret(textareaRef.current, pos);
    }, [value, applyCaret]);

    useImperativeHandle(
      ref,
      () => ({
        insertAtCursor: (text: string) => {
          const textarea = textareaRef.current;
          if (!textarea) return;

          const current = textarea.value;
          const start = textarea.selectionStart;
          const end = textarea.selectionEnd;
          const newValue =
            current.substring(0, start) + text + current.substring(end);

          setAutocompleteArmed(false);
          if (newValue === current) {
            applyCaret(textarea, start + text.length);
            return;
          }
          pendingCaretRef.current = start + text.length;
          onChange(newValue);
        },
        focus: () => {
          textareaRef.current?.focus();
        },
      }),
      [onChange, applyCaret],
    );

    const handleScroll = useCallback(() => {
      if (textareaRef.current && highlightRef.current) {
        highlightRef.current.scrollTop = textareaRef.current.scrollTop;
        highlightRef.current.scrollLeft = textareaRef.current.scrollLeft;
      }
      if (textareaRef.current && errorOverlayRef.current) {
        errorOverlayRef.current.scrollTop = textareaRef.current.scrollTop;
        errorOverlayRef.current.scrollLeft = textareaRef.current.scrollLeft;
      }
    }, []);

    // Every keydown is offered to the dropdown first; it only claims keys while
    // it is actually showing something, so Enter/Tab/arrows behave normally the
    // rest of the time — including elsewhere in the dialog, which a
    // document-level listener would have hijacked.
    const handleKeyDown = useCallback(
      (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        autocompleteRef.current?.handleKeyDown(e);
      },
      [],
    );

    const handleChange = useCallback(
      (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        setCaret(e.target.selectionStart);
        // Any edit re-arms the dropdown, so backspacing back to a prefix that
        // matches again brings the suggestions back.
        setAutocompleteArmed(true);
        onChange(e.target.value);
      },
      [onChange],
    );

    const lastSyncRef = useRef({ value, caret: 0 });
    const handleSelectionChange = useCallback(() => {
      const textarea = textareaRef.current;
      if (!textarea) return;

      const pos = textarea.selectionStart;
      const previous = lastSyncRef.current;
      const textChanged = textarea.value !== previous.value;
      const caretMoved = pos !== previous.caret;
      lastSyncRef.current = { value: textarea.value, caret: pos };

      setCaret(pos);
      if (!textChanged && caretMoved) setAutocompleteArmed(false);
    }, []);

    const handleSuggestionSelect = useCallback(
      (suggestion: AutocompleteSuggestion, context: CompletionContext) => {
        const textarea = textareaRef.current;
        if (!textarea) return;

        const text = textarea.value;
        const insertText = suggestion.insertText;
        const cursorPlaceholder = "{{cursor}}";
        const hasCursorPlaceholder = insertText.includes(cursorPlaceholder);
        const finalText = hasCursorPlaceholder
          ? insertText.replace(cursorPlaceholder, "")
          : insertText;

        // Replaces exactly the span the suggestion was matched against, so
        // accepting mid-word cannot duplicate the tail.
        const newText =
          text.substring(0, context.start) +
          finalText +
          text.substring(context.end);

        const newCaret = hasCursorPlaceholder
          ? context.start + insertText.indexOf(cursorPlaceholder)
          : context.start + finalText.length;

        setAutocompleteArmed(
          context.kind === "general" &&
            getCompletionContext(newText, newCaret).kind !== "general",
        );

        if (newText === text) {
          applyCaret(textarea, newCaret);
          return;
        }
        pendingCaretRef.current = newCaret;
        onChange(newText);
      },
      [onChange, applyCaret],
    );

    const handleDismiss = useCallback(() => setAutocompleteArmed(false), []);

    const createErrorOverlay = useCallback(() => {
      if (!value) return "";
      if (validationErrors.length === 0) {
        return value.replace(/./g, '<span class="transparent-char">$&</span>');
      }

      const lines = value.split("\n");
      const overlayParts: string[] = [];

      for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
        const line = lines[lineIndex];
        const lineErrors = validationErrors.filter(
          (err) => err.line === lineIndex,
        );

        if (lineErrors.length === 0) {
          overlayParts.push(
            line.replace(/./g, '<span class="transparent-char">$&</span>'),
          );
        } else {
          let processedLine = "";
          for (let charIndex = 0; charIndex < line.length; charIndex++) {
            const char = line[charIndex];
            const charErrors = lineErrors.filter(
              (err) =>
                charIndex >= err.column && charIndex < err.column + err.length,
            );

            const escapedChar = char
              .replace(/&/g, "&amp;")
              .replace(/</g, "&lt;")
              .replace(/>/g, "&gt;")
              .replace(/"/g, "&quot;")
              .replace(/'/g, "&#39;");

            if (charErrors.length > 0) {
              const error = charErrors[0];
              const severity = error.severity === "error" ? "error" : "warning";
              const escapedMessage = error.message
                .replace(/&/g, "&amp;")
                .replace(/"/g, "&quot;")
                .replace(/'/g, "&#39;");
              processedLine += `<span class="validation-${severity}" data-error="${escapedMessage}" title="${escapedMessage}">${escapedChar}</span>`;
            } else {
              processedLine += `<span class="transparent-char">${escapedChar}</span>`;
            }
          }
          overlayParts.push(processedLine);
        }
      }

      return overlayParts.join("\n");
    }, [value, validationErrors]);

    useEffect(() => {
      if (validationTimeoutRef.current) {
        clearTimeout(validationTimeoutRef.current);
      }

      if (!value.trim()) {
        setValidationErrors([]);
        validationTimeoutRef.current = null;
        return;
      }

      const timeoutId = setTimeout(() => {
        const errors = validateFlowExprCode(value);
        setValidationErrors(errors);
        validationTimeoutRef.current = null;
      }, 300);

      validationTimeoutRef.current = timeoutId;

      return () => {
        if (timeoutId) clearTimeout(timeoutId);
      };
    }, [value]);

    useEffect(() => {
      const syncStyles = () => {
        if (
          textareaRef.current &&
          (highlightRef.current ||
            placeholderRef.current ||
            errorOverlayRef.current)
        ) {
          const textarea = textareaRef.current;
          const computedStyle = window.getComputedStyle(textarea);

          const stylesToCopy = [
            "fontSize",
            "fontFamily",
            "fontWeight",
            "lineHeight",
            "letterSpacing",
            "wordSpacing",
            "tabSize",
            "textIndent",
            "textTransform",
            "textAlign",
            "padding",
            "paddingTop",
            "paddingRight",
            "paddingBottom",
            "paddingLeft",
            "border",
            "borderWidth",
            "borderStyle",
            "borderColor",
            "borderTop",
            "borderRight",
            "borderBottom",
            "borderLeft",
            "margin",
            "marginTop",
            "marginRight",
            "marginBottom",
            "marginLeft",
            "boxSizing",
            "width",
            "fontStretch",
            "fontSizeAdjust",
            "fontVariant",
            "fontKerning",
            "textRendering",
            "textDecorationSkipInk",
          ];

          if (highlightRef.current) {
            const highlight = highlightRef.current;
            stylesToCopy.forEach((prop) => {
              (highlight.style as any)[prop] = computedStyle.getPropertyValue(
                prop.replace(/([A-Z])/g, "-$1").toLowerCase(),
              );
            });
            highlight.style.position = "absolute";
            highlight.style.top = "0";
            highlight.style.left = "0";
            highlight.style.pointerEvents = "none";
            highlight.style.overflow = "hidden";
            highlight.style.whiteSpace = "pre-wrap";
            highlight.style.overflowWrap = "break-word";
          }

          if (errorOverlayRef.current) {
            const errorOverlay = errorOverlayRef.current;
            stylesToCopy.forEach((prop) => {
              (errorOverlay.style as any)[prop] =
                computedStyle.getPropertyValue(
                  prop.replace(/([A-Z])/g, "-$1").toLowerCase(),
                );
            });
            errorOverlay.style.position = "absolute";
            errorOverlay.style.top = "0";
            errorOverlay.style.left = "0";
            errorOverlay.style.pointerEvents = "auto";
            errorOverlay.style.overflow = "hidden";
            errorOverlay.style.whiteSpace = "pre-wrap";
            errorOverlay.style.overflowWrap = "break-word";
          }

          if (placeholderRef.current) {
            const ph = placeholderRef.current;
            stylesToCopy.forEach((prop) => {
              (ph.style as any)[prop] = computedStyle.getPropertyValue(
                prop.replace(/([A-Z])/g, "-$1").toLowerCase(),
              );
            });
            ph.style.position = "absolute";
            ph.style.top = "0px";
            ph.style.left = "0px";
            ph.style.pointerEvents = "none";
            ph.style.overflow = "hidden";
            ph.style.whiteSpace = "pre-wrap";
            ph.style.overflowWrap = "break-word";
            ph.style.color = "rgb(107 114 128)";
          }
        }
      };

      syncStyles();

      const resizeObserver = new ResizeObserver(syncStyles);
      if (textareaRef.current) {
        resizeObserver.observe(textareaRef.current);
      }

      return () => resizeObserver.disconnect();
    }, [value]);

    return (
      <div className={`relative ${className} flex`}>
        <style>{`
        .transparent-char {
          background: transparent !important;
          color: transparent !important;
          border: none !important;
        }
        .validation-error {
          background-color: rgba(254, 226, 226, 0.8) !important;
          border-bottom: 2px solid #dc2626 !important;
          color: inherit !important;
        }
        .dark .validation-error {
          background-color: rgba(69, 10, 10, 0.8) !important;
          border-bottom: 2px solid #dc2626 !important;
        }
        .validation-warning {
          background-color: rgba(254, 243, 199, 0.8) !important;
          border-bottom: 2px solid #d97706 !important;
          color: inherit !important;
        }
        .dark .validation-warning {
          background-color: rgba(69, 26, 3, 0.8) !important;
          border-bottom: 2px solid #d97706 !important;
        }
        .validation-error, .validation-warning {
          transition: background-color 0.1s ease;
          pointer-events: auto;
        }
      `}</style>

        <TextArea
          ref={textareaRef}
          className="relative max-h-full flex-1 resize-none rounded-none border-transparent text-transparent caret-gray-900 selection:bg-blue-200 focus-visible:ring-0 dark:caret-gray-100 dark:selection:bg-logo/25"
          style={{ zIndex: 3 }}
          placeholder=""
          value={value}
          onChange={handleChange}
          onSelect={handleSelectionChange}
          onScroll={handleScroll}
          onKeyDown={handleKeyDown}
          spellCheck={false}
          {...props}
        />

        <div
          ref={errorOverlayRef}
          className="pointer-events-none absolute h-full bg-transparent"
          style={{ zIndex: 4 }}
          dangerouslySetInnerHTML={{ __html: createErrorOverlay() }}
        />

        <div
          ref={highlightRef}
          className="pointer-events-none absolute h-full bg-transparent"
          style={{ zIndex: 1 }}>
          <FlowExprSyntaxHighlighter code={value} className="" />
        </div>

        {!value && placeholder && (
          <div
            ref={placeholderRef}
            className="pointer-events-none absolute text-muted-foreground"
            style={{ zIndex: 0, top: 0, left: 0 }}>
            {placeholder}
          </div>
        )}

        {validationErrors.length > 0 && (
          <div className="absolute bottom-2 left-2 flex items-center gap-2 text-xs">
            {validationErrors.filter((err) => err.severity === "error").length >
              0 && (
              <span className="flex items-center gap-1 text-red-600 dark:text-red-400">
                <span>❌</span>
                {
                  validationErrors.filter((err) => err.severity === "error")
                    .length
                }{" "}
                error(s)
              </span>
            )}
            {validationErrors.filter((err) => err.severity === "warning")
              .length > 0 && (
              <span className="flex items-center gap-1 text-amber-600 dark:text-amber-400">
                <span>⚠️</span>
                {
                  validationErrors.filter((err) => err.severity === "warning")
                    .length
                }{" "}
                warning(s)
              </span>
            )}
          </div>
        )}

        <FlowExprAutocomplete
          ref={autocompleteRef}
          textareaRef={textareaRef}
          value={value}
          caret={caret}
          open={autocompleteArmed}
          onSuggestionSelect={handleSuggestionSelect}
          onDismiss={handleDismiss}
          attributeSuggestions={attributeSuggestions}
          variableSuggestions={variableSuggestions}
        />
      </div>
    );
  },
);

FlowExprCodeEditor.displayName = "FlowExprCodeEditor";

export default FlowExprCodeEditor;
