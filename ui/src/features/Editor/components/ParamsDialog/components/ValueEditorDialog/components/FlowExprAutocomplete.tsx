import { useCallback, useEffect, useState, useRef, useMemo } from "react";

import { useT } from "@flow/lib/i18n";
import { AttrType } from "@flow/types/schemaPreview";

import { isInsideAttributeAccessor } from "./flowExprAttributeContext";
import {
  AutocompleteSuggestion,
  getFlowExprAutocompleteSuggestions,
  TYPE_COLOR,
} from "./flowExprConstants";

type Props = {
  textareaRef: React.RefObject<HTMLTextAreaElement | null>;
  value: string;
  onSuggestionSelect: (suggestion: AutocompleteSuggestion) => void;
  visible: boolean;
  onVisibilityChange: (visible: boolean) => void;
  // Per-node attribute-name suggestions, shown when the cursor is inside an
  // `attributes["…"]` accessor. Sourced from probed reader schemas.
  attributeSuggestions?: AutocompleteSuggestion[];
};

const FlowExprAutocomplete: React.FC<Props> = ({
  textareaRef,
  value,
  onSuggestionSelect,
  visible,
  onVisibilityChange,
  attributeSuggestions,
}) => {
  const t = useT();
  const [suggestions, setSuggestions] = useState<AutocompleteSuggestion[]>([]);
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

  const getCurrentWordAndPosition = useCallback(() => {
    if (!textareaRef.current) return { word: "", start: 0, end: 0 };

    const textarea = textareaRef.current;
    const cursorPos = textarea.selectionStart;
    const text = textarea.value;

    let start = cursorPos;
    let end = cursorPos;

    while (start > 0 && /[a-zA-Z0-9_:.]/.test(text[start - 1])) {
      start--;
    }

    while (end < text.length && /[a-zA-Z0-9_]/.test(text[end])) {
      end++;
    }

    const word = text.substring(start, end);
    return { word, start, end };
  }, [textareaRef]);

  const getFilteredSuggestions = useCallback(
    (word: string): AutocompleteSuggestion[] => {
      if (word.length < 1) return [];

      const lowerWord = word.toLowerCase();
      const dotIndex = lowerWord.lastIndexOf(".");
      const hasDot = dotIndex >= 0;
      // After a dot, match only the suffix (the method name being typed).
      const matchWord = hasDot ? lowerWord.substring(dotIndex + 1) : lowerWord;

      let candidates: AutocompleteSuggestion[];
      if (hasDot) {
        if (matchWord.length === 0) {
          candidates = functionSuggestions;
        } else {
          const firstChar = matchWord.charAt(0);
          candidates = (indexedSuggestions.get(firstChar) || []).filter(
            (s) => s.type === "function" || s.type === "variable",
          );
        }
      } else {
        const firstChar = lowerWord.charAt(0);
        candidates = indexedSuggestions.get(firstChar) || [];
      }

      const filtered = candidates.filter((suggestion) =>
        suggestion.label.toLowerCase().startsWith(matchWord),
      );

      return filtered.sort((a, b) => {
        const aExact = a.label.toLowerCase() === matchWord ? 0 : 1;
        const bExact = b.label.toLowerCase() === matchWord ? 0 : 1;
        if (aExact !== bExact) return aExact - bExact;

        const typePriority: Record<string, number> = {
          keyword: 0,
          function: 1,
          variable: 2,
          operator: 3,
        };
        return (typePriority[a.type] || 5) - (typePriority[b.type] || 5);
      });
    },
    [indexedSuggestions, functionSuggestions],
  );

  const cursorInsideAttributeAccessor = useCallback(() => {
    const textarea = textareaRef.current;
    if (!textarea) return false;
    const before = textarea.value.substring(0, textarea.selectionStart);
    return isInsideAttributeAccessor(before);
  }, [textareaRef]);

  const getFilteredAttributeSuggestions = useCallback(
    (word: string): AutocompleteSuggestion[] => {
      const candidates = attributeSuggestions ?? [];
      if (candidates.length === 0) return [];
      const lowerWord = word.toLowerCase();
      if (lowerWord.length === 0) return candidates;
      return candidates.filter((suggestion) =>
        suggestion.label.toLowerCase().startsWith(lowerWord),
      );
    },
    [attributeSuggestions],
  );

  const calculatePosition = useCallback(() => {
    if (!textareaRef.current) return;

    const textarea = textareaRef.current;
    const { start } = getCurrentWordAndPosition();
    const computedStyle = window.getComputedStyle(textarea);

    const paddingLeft = parseInt(computedStyle.paddingLeft) || 0;
    const paddingTop = parseInt(computedStyle.paddingTop) || 0;
    const scrollTop = textarea.scrollTop;
    const scrollLeft = textarea.scrollLeft;

    const textBeforeCursor = textarea.value.substring(0, start);
    const lines = textBeforeCursor.split("\n");
    const currentLineText = lines[lines.length - 1];
    const lineNumber = lines.length - 1;

    const lineHeight = parseInt(computedStyle.lineHeight);
    const actualLineHeight = isNaN(lineHeight)
      ? parseInt(computedStyle.fontSize) * 1.2
      : lineHeight;

    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");
    if (ctx) {
      ctx.font = `${computedStyle.fontSize} ${computedStyle.fontFamily}`;
      const textWidth = ctx.measureText(currentLineText).width;

      setPosition({
        top:
          paddingTop +
          lineNumber * actualLineHeight -
          scrollTop +
          actualLineHeight,
        left: paddingLeft + textWidth - scrollLeft,
      });
    }
  }, [textareaRef, getCurrentWordAndPosition]);

  useEffect(() => {
    if (!visible) return;

    const { word } = getCurrentWordAndPosition();
    const filtered = cursorInsideAttributeAccessor()
      ? getFilteredAttributeSuggestions(word)
      : getFilteredSuggestions(word);

    setSuggestions(filtered);
    setSelectedIndex(0);

    if (filtered.length > 0) {
      calculatePosition();
    } else {
      onVisibilityChange(false);
    }
  }, [
    value,
    visible,
    getCurrentWordAndPosition,
    getFilteredSuggestions,
    cursorInsideAttributeAccessor,
    getFilteredAttributeSuggestions,
    calculatePosition,
    onVisibilityChange,
  ]);

  // Scroll selected item into view when navigating with arrow keys.
  useEffect(() => {
    if (!containerRef.current) return;
    const selectedEl = containerRef.current.children[selectedIndex];
    if (selectedEl) selectedEl.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  // Handle keyboard navigation via document listener. Arrow/Enter/Tab bubble
  // here naturally; ESC is handled separately in the editor (capture phase).
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!visible || suggestions.length === 0) return;

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((prev) => (prev + 1) % suggestions.length);
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex(
            (prev) => (prev - 1 + suggestions.length) % suggestions.length,
          );
          break;
        case "Enter":
        case "Tab":
          e.preventDefault();
          if (suggestions[selectedIndex]) {
            onSuggestionSelect(suggestions[selectedIndex]);
          }
          break;
        case "Escape":
          e.preventDefault();
          onVisibilityChange(false);
          break;
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [
    visible,
    suggestions,
    selectedIndex,
    onSuggestionSelect,
    onVisibilityChange,
  ]);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        visible &&
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        onVisibilityChange(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [visible, onVisibilityChange]);

  if (!visible || suggestions.length === 0) {
    return null;
  }

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
          className={`cursor-pointer px-3 py-2 text-sm ${
            index === selectedIndex
              ? "bg-accent text-accent-foreground"
              : "hover:bg-accent/50"
          }`}
          onClick={() => onSuggestionSelect(suggestion)}>
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
              className={`mt-1 font-mono text-xs ${TYPE_COLOR[suggestion.detail as AttrType]}`}>
              {suggestion.detail}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

export default FlowExprAutocomplete;
