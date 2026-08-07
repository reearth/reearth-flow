import {
  useCallback,
  useEffect,
  useState,
  useRef,
  useMemo,
  useImperativeHandle,
  forwardRef,
  useLayoutEffect,
} from "react";

import { useT } from "@flow/lib/i18n";
import { AttrType } from "@flow/types/schemaPreview";

import {
  getCompletionContext,
  type CompletionContext,
} from "./flowExprAttributeContext";
import {
  AutocompleteSuggestion,
  getFlowExprAutocompleteSuggestions,
  TYPE_COLOR,
} from "./flowExprConstants";

export type FlowExprAutocompleteRef = {
  handleKeyDown: (e: React.KeyboardEvent<HTMLTextAreaElement>) => boolean;
};

type Props = {
  textareaRef: React.RefObject<HTMLTextAreaElement | null>;
  value: string;
  caret: number;
  open: boolean;
  onSuggestionSelect: (
    suggestion: AutocompleteSuggestion,
    context: CompletionContext,
  ) => void;
  onDismiss: () => void;
  // Per-node attribute-name suggestions, shown when the cursor is inside an
  // `attributes["…"]` accessor. Sourced from probed reader schemas.
  attributeSuggestions?: AutocompleteSuggestion[];
  // Workflow-variable names, shown when the cursor is inside an `variables["…"]`
  // lookup. Sourced from the project's workflow variables.
  variableSuggestions?: AutocompleteSuggestion[];
};

const TYPE_PRIORITY: Record<string, number> = {
  keyword: 0,
  function: 1,
  variable: 2,
  operator: 3,
};

const NAVIGATION_KEYS = ["ArrowUp", "ArrowDown", "Enter", "Tab"];

const FlowExprAutocomplete = forwardRef<FlowExprAutocompleteRef, Props>(
  (
    {
      textareaRef,
      value,
      caret,
      open,
      onSuggestionSelect,
      onDismiss,
      attributeSuggestions,
      variableSuggestions,
    },
    ref,
  ) => {
    const t = useT();
    const [selectedIndex, setSelectedIndex] = useState(0);
    const [position, setPosition] = useState({ top: 0, left: 0 });
    const containerRef = useRef<HTMLDivElement>(null);

    const { indexedSuggestions, functionSuggestions } = useMemo(() => {
      const index = new Map<string, AutocompleteSuggestion[]>();
      const functions: AutocompleteSuggestion[] = [];
      const allSuggestions = getFlowExprAutocompleteSuggestions(t);

      allSuggestions.forEach((suggestion) => {
        const firstChar = suggestion.label.charAt(0).toLowerCase();
        if (!index.has(firstChar)) index.set(firstChar, []);
        const bucket = index.get(firstChar);
        if (bucket) bucket.push(suggestion);
        if (suggestion.type === "function" || suggestion.type === "variable")
          functions.push(suggestion);
      });

      return { indexedSuggestions: index, functionSuggestions: functions };
    }, [t]);

    const context = useMemo(
      () => getCompletionContext(value, caret),
      [value, caret],
    );

    const getGeneralSuggestions = useCallback(
      ({ prefix, afterDot }: CompletionContext): AutocompleteSuggestion[] => {
        if (prefix.length < 1 && !afterDot) return [];

        const matchWord = prefix.toLowerCase();

        let candidates: AutocompleteSuggestion[];
        if (afterDot) {
          candidates =
            matchWord.length === 0
              ? functionSuggestions
              : (indexedSuggestions.get(matchWord.charAt(0)) || []).filter(
                  (s) => s.type === "function" || s.type === "variable",
                );
        } else {
          candidates = indexedSuggestions.get(matchWord.charAt(0)) || [];
        }

        return candidates
          .filter((suggestion) =>
            suggestion.label.toLowerCase().startsWith(matchWord),
          )
          .sort((a, b) => {
            const aExact = a.label.toLowerCase() === matchWord ? 0 : 1;
            const bExact = b.label.toLowerCase() === matchWord ? 0 : 1;
            if (aExact !== bExact) return aExact - bExact;
            return (TYPE_PRIORITY[a.type] ?? 5) - (TYPE_PRIORITY[b.type] ?? 5);
          });
      },
      [indexedSuggestions, functionSuggestions],
    );

    const suggestions = useMemo(() => {
      if (!open) return [];

      if (context.kind === "general") return getGeneralSuggestions(context);

      // Quoted-key accessors: `attributes["…"]` and `variables["…"]` behave the same
      // way, differing only in where the candidate names come from.
      const candidates =
        (context.kind === "attribute"
          ? attributeSuggestions
          : variableSuggestions) ?? [];
      const prefix = context.prefix.toLowerCase();
      // An empty prefix right after the opening quote lists every candidate.
      return prefix.length === 0
        ? candidates
        : candidates.filter((suggestion) =>
            suggestion.label.toLowerCase().startsWith(prefix),
          );
    }, [
      open,
      context,
      attributeSuggestions,
      variableSuggestions,
      getGeneralSuggestions,
    ]);

    const visible = open && suggestions.length > 0;

    // Typing moves the highlight back to the best match. `context` is the only
    // user-driven input to the list, so that is what the reset keys on.
    useEffect(() => {
      setSelectedIndex(0);
    }, [context]);

    // `attributeSuggestions` can also change under us when a schema probe
    // lands. That should not yank the highlight, only keep it in range.
    const activeIndex = Math.min(selectedIndex, suggestions.length - 1);

    useLayoutEffect(() => {
      if (!visible || !textareaRef.current) return;
      const textarea = textareaRef.current;

      const measure = () => {
        const computedStyle = window.getComputedStyle(textarea);

        const paddingLeft = parseInt(computedStyle.paddingLeft) || 0;
        const paddingTop = parseInt(computedStyle.paddingTop) || 0;

        const lines = textarea.value.substring(0, context.start).split("\n");
        const currentLineText = lines[lines.length - 1];
        const lineNumber = lines.length - 1;

        const lineHeight = parseInt(computedStyle.lineHeight);
        const actualLineHeight = isNaN(lineHeight)
          ? parseInt(computedStyle.fontSize) * 1.2
          : lineHeight;

        const ctx = document.createElement("canvas").getContext("2d");
        if (!ctx) return;
        ctx.font = `${computedStyle.fontSize} ${computedStyle.fontFamily}`;

        setPosition({
          top:
            paddingTop +
            (lineNumber + 1) * actualLineHeight -
            textarea.scrollTop,
          left:
            paddingLeft +
            ctx.measureText(currentLineText).width -
            textarea.scrollLeft,
        });
      };

      measure();
      textarea.addEventListener("scroll", measure);
      return () => textarea.removeEventListener("scroll", measure);
    }, [visible, context.start, textareaRef]);

    // Scroll selected item into view when navigating with arrow keys.
    useEffect(() => {
      if (!containerRef.current) return;
      const selectedEl = containerRef.current.children[activeIndex];
      if (selectedEl) selectedEl.scrollIntoView({ block: "nearest" });
    }, [activeIndex]);

    useImperativeHandle(
      ref,
      () => ({
        handleKeyDown: (e) => {
          if (!visible || !NAVIGATION_KEYS.includes(e.key)) return false;

          e.preventDefault();
          switch (e.key) {
            case "ArrowDown":
              setSelectedIndex((activeIndex + 1) % suggestions.length);
              break;
            case "ArrowUp":
              setSelectedIndex(
                (activeIndex - 1 + suggestions.length) % suggestions.length,
              );
              break;
            default: {
              const suggestion = suggestions[activeIndex];
              if (suggestion) onSuggestionSelect(suggestion, context);
              break;
            }
          }
          return true;
        },
      }),
      [visible, suggestions, activeIndex, onSuggestionSelect, context],
    );

    // Capture phase so ESC closes the dropdown before the Dialog sees it — but
    // only while something is actually shown, otherwise ESC could never close
    // the dialog itself.
    useEffect(() => {
      if (!visible) return;
      const handleEsc = (e: KeyboardEvent) => {
        if (e.key !== "Escape") return;
        e.stopImmediatePropagation();
        e.preventDefault();
        onDismiss();
      };
      document.addEventListener("keydown", handleEsc, { capture: true });
      return () =>
        document.removeEventListener("keydown", handleEsc, { capture: true });
    }, [visible, onDismiss]);

    useEffect(() => {
      if (!visible) return;
      const handleClickOutside = (e: MouseEvent) => {
        if (
          containerRef.current &&
          !containerRef.current.contains(e.target as Node)
        ) {
          onDismiss();
        }
      };
      document.addEventListener("mousedown", handleClickOutside);
      return () =>
        document.removeEventListener("mousedown", handleClickOutside);
    }, [visible, onDismiss]);

    if (!visible) return null;

    const getTypeColor = (type: string): string => {
      switch (type) {
        case "keyword":
          return "text-purple-600 dark:text-purple-400";
        case "function":
          return "text-blue-600 dark:text-blue-400";
        case "variable":
          return "text-green-600 dark:text-green-400";
        case "operator":
          return "text-red-600 dark:text-red-400";
        case "attribute":
          return "text-yellow-600 dark:text-yellow-400";
        default:
          return "text-gray-600 dark:text-gray-400";
      }
    };

    return (
      <div
        ref={containerRef}
        className="absolute z-50 max-h-64 w-90 overflow-auto rounded-lg border bg-popover/70 shadow-lg"
        style={{ top: position.top, left: position.left }}>
        {suggestions.map((suggestion, index) => (
          <div
            key={`${suggestion.label}-${index}`}
            data-testid="editor-suggestion"
            className={`cursor-pointer px-3 py-2 text-sm ${
              index === activeIndex
                ? "bg-accent text-accent-foreground"
                : "hover:bg-accent/50"
            }`}
            // mousedown, so the choice lands before the textarea blurs.
            onMouseDown={(e) => {
              e.preventDefault();
              onSuggestionSelect(suggestion, context);
            }}>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span
                  className={`rounded px-1 py-0.5 font-mono text-xs ${getTypeColor(suggestion.type)} bg-current/10`}>
                  {suggestion.type}
                </span>
                <span className="font-medium">{suggestion.label}</span>
              </div>
            </div>
            {suggestion.description && (
              <div className="mt-1 text-xs text-muted-foreground">
                {suggestion.description}
              </div>
            )}
            {suggestion.detail && (
              <div
                className={`mt-1 font-mono text-xs ${
                  TYPE_COLOR[suggestion.detail as AttrType] ??
                  "text-muted-foreground"
                }`}>
                {suggestion.detail}
              </div>
            )}
          </div>
        ))}
      </div>
    );
  },
);

FlowExprAutocomplete.displayName = "FlowExprAutocomplete";

export default FlowExprAutocomplete;
