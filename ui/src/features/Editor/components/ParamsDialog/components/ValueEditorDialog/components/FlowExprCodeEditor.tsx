import {
  useCallback,
  useRef,
  useEffect,
  useState,
  useImperativeHandle,
  forwardRef,
} from "react";

import { TextArea } from "@flow/components";

import { type AutocompleteSuggestion } from "./constants";
import FlowExprAutocomplete from "./FlowExprAutocomplete";
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
  "data-testid"?: string;
  "aria-label"?: string;
  "data-placeholder"?: string;
};

const FlowExprCodeEditor = forwardRef<FlowExprCodeEditorRef, Props>(
  ({ value = "", onChange, placeholder, className = "", ...props }, ref) => {
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const highlightRef = useRef<HTMLDivElement>(null);
    const placeholderRef = useRef<HTMLDivElement>(null);
    const errorOverlayRef = useRef<HTMLDivElement>(null);

    const [autocompleteVisible, setAutocompleteVisible] = useState(false);
    const autocompleteVisibleRef = useRef(false);
    autocompleteVisibleRef.current = autocompleteVisible;

    const [validationErrors, setValidationErrors] = useState<ValidationError[]>(
      [],
    );
    const validationTimeoutRef = useRef<NodeJS.Timeout | null>(null);

    // Capture-phase ESC handler — fires before Radix Dialog's bubbling handler
    // so ESC only closes the autocomplete when it is open, not the dialog.
    useEffect(() => {
      const handleEsc = (e: KeyboardEvent) => {
        if (e.key === "Escape" && autocompleteVisibleRef.current) {
          e.stopImmediatePropagation();
          setAutocompleteVisible(false);
        }
      };
      document.addEventListener("keydown", handleEsc, { capture: true });
      return () =>
        document.removeEventListener("keydown", handleEsc, { capture: true });
    }, []);

    useImperativeHandle(
      ref,
      () => ({
        insertAtCursor: (text: string) => {
          if (!textareaRef.current) return;

          const textarea = textareaRef.current;
          const start = textarea.selectionStart;
          const end = textarea.selectionEnd;

          const newValue =
            value.substring(0, start) + text + value.substring(end);
          onChange(newValue);

          setTimeout(() => {
            const newCursorPos = start + text.length;
            textarea.setSelectionRange(newCursorPos, newCursorPos);
            textarea.focus();
          }, 10);
        },
        focus: () => {
          textareaRef.current?.focus();
        },
      }),
      [value, onChange],
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

    const handleKeyDown = useCallback(
      (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (
          autocompleteVisible &&
          ["ArrowUp", "ArrowDown", "Enter", "Tab", "Escape"].includes(e.key)
        ) {
          // Let the event bubble to the autocomplete's document listener.
          // ESC is intercepted separately via a capture-phase handler.
          return;
        }

        if (/^[a-zA-Z0-9_:.]$/.test(e.key)) {
          setTimeout(() => setAutocompleteVisible(true), 10);
        } else if (e.key === "Backspace" || e.key === "Delete") {
          setTimeout(() => {
            const textarea = textareaRef.current;
            if (textarea) {
              const cursorPos = textarea.selectionStart;
              const textBeforeCursor = textarea.value.substring(0, cursorPos);
              const lastWord =
                textBeforeCursor.split(/[^a-zA-Z0-9_:.]/).pop() || "";
              if (lastWord.length < 1) {
                setAutocompleteVisible(false);
              }
            }
          }, 10);
        } else if (
          e.key === " " ||
          e.key === "(" ||
          e.key === ")" ||
          e.key === ";" ||
          e.key === ","
        ) {
          setAutocompleteVisible(false);
        }
      },
      [autocompleteVisible],
    );

    const handleSuggestionSelect = useCallback(
      (suggestion: AutocompleteSuggestion) => {
        if (!textareaRef.current) return;

        const textarea = textareaRef.current;
        const cursorPos = textarea.selectionStart;
        const text = textarea.value;

        let start = cursorPos;
        while (start > 0 && /[a-zA-Z0-9_:.]/.test(text[start - 1])) {
          start--;
        }

        const word = text.substring(start, cursorPos);
        const lastDot = word.lastIndexOf(".");
        const replaceStart = lastDot >= 0 ? start + lastDot + 1 : start;

        const insertText = suggestion.insertText;
        const cursorPlaceholder = "{{cursor}}";
        const hasCursorPlaceholder = insertText.includes(cursorPlaceholder);
        const finalText = hasCursorPlaceholder
          ? insertText.replace(cursorPlaceholder, "")
          : insertText;

        const newText =
          text.substring(0, replaceStart) +
          finalText +
          text.substring(cursorPos);
        onChange(newText);

        setTimeout(() => {
          if (hasCursorPlaceholder) {
            const placeholderPos =
              replaceStart + insertText.indexOf(cursorPlaceholder);
            textarea.setSelectionRange(placeholderPos, placeholderPos);
          } else {
            const newCursorPos = replaceStart + finalText.length;
            textarea.setSelectionRange(newCursorPos, newCursorPos);
          }
          textarea.focus();
        }, 10);

        setAutocompleteVisible(false);
      },
      [onChange],
    );

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
          onChange={(e) => onChange(e.target.value)}
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
          textareaRef={textareaRef}
          value={value}
          onSuggestionSelect={handleSuggestionSelect}
          visible={autocompleteVisible}
          onVisibilityChange={setAutocompleteVisible}
        />
      </div>
    );
  },
);

FlowExprCodeEditor.displayName = "FlowExprCodeEditor";

export default FlowExprCodeEditor;
