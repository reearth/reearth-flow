import { useCallback, useRef, useEffect, useState, useImperativeHandle, forwardRef } from "react";

import { TextArea } from "@flow/components";

import { type AutocompleteSuggestion } from "./constants";
import RhaiAutocomplete from "./RhaiAutocomplete";
import RhaiSyntaxHighlighter from "./RhaiSyntaxHighlighter";
import { validateRhaiCode, type ValidationError } from "./RhaiValidator";

export type RhaiCodeEditorRef = {
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

const RhaiCodeEditor = forwardRef<RhaiCodeEditorRef, Props>(({
  value,
  onChange,
  placeholder,
  className = "",
  ...props
}, ref) => {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const highlightRef = useRef<HTMLDivElement>(null);
  const placeholderRef = useRef<HTMLDivElement>(null);
  const errorOverlayRef = useRef<HTMLDivElement>(null);

  // Autocomplete state
  const [autocompleteVisible, setAutocompleteVisible] = useState(false);

  // Validation state with debounced validation
  const [validationErrors, setValidationErrors] = useState<ValidationError[]>(
    [],
  );
  const validationTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Expose methods via ref
  useImperativeHandle(ref, () => ({
    insertAtCursor: (text: string) => {
      if (!textareaRef.current) return;
      
      const textarea = textareaRef.current;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const currentValue = value;
      
      // Insert text at cursor position
      const newValue = currentValue.substring(0, start) + text + currentValue.substring(end);
      onChange(newValue);
      
      // Set cursor position after inserted text
      setTimeout(() => {
        const newCursorPos = start + text.length;
        textarea.setSelectionRange(newCursorPos, newCursorPos);
        textarea.focus();
      }, 10);
    },
    focus: () => {
      textareaRef.current?.focus();
    }
  }), [value, onChange]);

  // Sync scroll position between textarea and highlight overlay
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

  // Handle autocomplete trigger
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      // Don't show autocomplete for navigation keys when it's already visible
      if (
        autocompleteVisible &&
        ["ArrowUp", "ArrowDown", "Enter", "Tab", "Escape"].includes(e.key)
      ) {
        return;
      }

      // Trigger autocomplete for typing letters, numbers, underscores, or namespace operators
      if (/^[a-zA-Z0-9_:]$/.test(e.key)) {
        // Small delay to let the character be added to the input first
        setTimeout(() => setAutocompleteVisible(true), 10);
      } else if (e.key === "Backspace" || e.key === "Delete") {
        // Keep autocomplete open when deleting, let it filter
        setTimeout(() => {
          const textarea = textareaRef.current;
          if (textarea) {
            const cursorPos = textarea.selectionStart;
            const textBeforeCursor = textarea.value.substring(0, cursorPos);
            const lastWord =
              textBeforeCursor.split(/[^a-zA-Z0-9_:.]/).pop() || "";

            // Close if no meaningful text to autocomplete
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
        // Close autocomplete for these characters
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

      // Find the current word boundaries
      let start = cursorPos;
      while (start > 0 && /[a-zA-Z0-9_:.]/.test(text[start - 1])) {
        start--;
      }

      // Replace current word with suggestion
      const newText =
        text.substring(0, start) +
        suggestion.insertText +
        text.substring(cursorPos);
      onChange(newText);

      // Set cursor position after the inserted text
      setTimeout(() => {
        const newCursorPos = start + suggestion.insertText.length;
        textarea.setSelectionRange(newCursorPos, newCursorPos);
        textarea.focus();
      }, 10);

      setAutocompleteVisible(false);
    },
    [onChange],
  );

  // Create error overlay content with optimized string building
  const createErrorOverlay = useCallback(() => {
    if (!value) return "";
    if (validationErrors.length === 0) {
      // If no errors, return transparent content efficiently
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
        // No errors in this line - process efficiently
        overlayParts.push(line.replace(/./g, '<span class="transparent-char">$&</span>'));
      } else {
        // Process line character by character only when there are errors
        let processedLine = "";
        for (let charIndex = 0; charIndex < line.length; charIndex++) {
          const char = line[charIndex];
          const charErrors = lineErrors.filter(
            (err) =>
              charIndex >= err.column && charIndex < err.column + err.length,
          );

          // Escape HTML characters
          const escapedChar = char
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#39;");

          if (charErrors.length > 0) {
            const error = charErrors[0]; // Use first error if multiple
            const severity = error.severity === "error" ? "error" : "warning";
            processedLine += `<span class="validation-${severity}" data-error="${encodeURIComponent(error.message)}" title="${encodeURIComponent(error.message)}">${escapedChar}</span>`;
          } else {
            processedLine += `<span class="transparent-char">${escapedChar}</span>`;
          }
        }
        overlayParts.push(processedLine);
      }
    }

    return overlayParts.join("\n");
  }, [value, validationErrors]);

  // Debounced validation to improve performance
  useEffect(() => {
    // Clear existing timeout
    if (validationTimeoutRef.current) {
      clearTimeout(validationTimeoutRef.current);
    }

    if (!value.trim()) {
      setValidationErrors([]);
      validationTimeoutRef.current = null;
      return;
    }

    // Set new timeout for validation
    const timeoutId = setTimeout(() => {
      const errors = validateRhaiCode(value);
      setValidationErrors(errors);
      validationTimeoutRef.current = null;
    }, 300); // 300ms debounce

    validationTimeoutRef.current = timeoutId;

    // Cleanup timeout on unmount
    return () => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
    };
  }, [value]);

  // Sync positioning and styles exactly
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

        // Copy ALL relevant styles to ensure perfect alignment
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
          // Additional text layout properties that might affect character positioning
          "fontStretch",
          "fontSizeAdjust",
          "fontVariant",
          "fontKerning",
          "textRendering",
          "textDecorationSkipInk",
        ];

        // Sync highlight overlay
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

        // Sync error overlay
        if (errorOverlayRef.current) {
          const errorOverlay = errorOverlayRef.current;
          stylesToCopy.forEach((prop) => {
            (errorOverlay.style as any)[prop] = computedStyle.getPropertyValue(
              prop.replace(/([A-Z])/g, "-$1").toLowerCase(),
            );
          });

          errorOverlay.style.position = "absolute";
          errorOverlay.style.top = "0";
          errorOverlay.style.left = "0";
          errorOverlay.style.pointerEvents = "auto"; // Allow hover for error tooltips
          errorOverlay.style.overflow = "hidden";
          errorOverlay.style.whiteSpace = "pre-wrap";
          errorOverlay.style.overflowWrap = "break-word";
        }

        // Sync placeholder
        if (placeholderRef.current) {
          const placeholder = placeholderRef.current;
          stylesToCopy.forEach((prop) => {
            (placeholder.style as any)[prop] = computedStyle.getPropertyValue(
              prop.replace(/([A-Z])/g, "-$1").toLowerCase(),
            );
          });

          placeholder.style.position = "absolute";
          placeholder.style.top = "0px";
          placeholder.style.left = "0px";
          placeholder.style.pointerEvents = "none";
          placeholder.style.overflow = "hidden";
          placeholder.style.whiteSpace = "pre-wrap";
          placeholder.style.overflowWrap = "break-word";
          placeholder.style.color = "rgb(107 114 128)"; // text-muted-foreground
        }
      }
    };

    // Sync immediately and on resize
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
        .validation-error:hover, .validation-warning:hover {
          cursor: help;
        }
      `}</style>

      {/* Invisible textarea for input */}
      <TextArea
        ref={textareaRef}
        className="relative max-h-full flex-1 resize-none rounded-none border-transparent text-transparent caret-gray-900 selection:bg-blue-200 focus-visible:ring-0 dark:caret-gray-100 dark:selection:bg-logo/25"
        style={{ zIndex: 3 }}
        placeholder="" // Remove duplicate placeholder
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onScroll={handleScroll}
        onKeyDown={handleKeyDown}
        spellCheck={false}
        {...props}
      />

      {/* Error highlighting overlay - positioned exactly over textarea */}
      <div
        ref={errorOverlayRef}
        className="pointer-events-auto absolute h-full bg-transparent"
        style={{ zIndex: 2 }}
        dangerouslySetInnerHTML={{ __html: createErrorOverlay() }}
      />

      {/* Syntax highlighted background - positioned exactly over textarea */}
      <div
        ref={highlightRef}
        className="pointer-events-none absolute h-full bg-transparent"
        style={{
          zIndex: 1,
        }}>
        <RhaiSyntaxHighlighter code={value} className="" />
      </div>

      {/* Single placeholder when empty */}
      {!value && placeholder && (
        <div
          ref={placeholderRef}
          className="pointer-events-none absolute text-muted-foreground"
          style={{
            zIndex: 0,
            top: 0,
            left: 0,
          }}>
          {placeholder}
        </div>
      )}

      {/* Validation error summary */}
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
          {validationErrors.filter((err) => err.severity === "warning").length >
            0 && (
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

      {/* Autocomplete dropdown */}
      <RhaiAutocomplete
        textareaRef={textareaRef}
        value={value}
        onSuggestionSelect={handleSuggestionSelect}
        visible={autocompleteVisible}
        onVisibilityChange={setAutocompleteVisible}
      />
    </div>
  );
});

RhaiCodeEditor.displayName = "RhaiCodeEditor";

export default RhaiCodeEditor;
