import { useCallback, useEffect, useState, useRef, useMemo } from "react";

import {
  RHAI_AUTOCOMPLETE_SUGGESTIONS,
  type AutocompleteSuggestion,
} from "./constants";

type Props = {
  textareaRef: React.RefObject<HTMLTextAreaElement | null>;
  value: string;
  onSuggestionSelect: (suggestion: AutocompleteSuggestion) => void;
  visible: boolean;
  onVisibilityChange: (visible: boolean) => void;
};

const RhaiAutocomplete: React.FC<Props> = ({
  textareaRef,
  value,
  onSuggestionSelect,
  visible,
  onVisibilityChange,
}) => {
  const [suggestions, setSuggestions] = useState<AutocompleteSuggestion[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  // Create indexed suggestions for faster searching
  const indexedSuggestions = useMemo(() => {
    const index = new Map<string, AutocompleteSuggestion[]>();
    
    // Group suggestions by first character for faster lookup
    RHAI_AUTOCOMPLETE_SUGGESTIONS.forEach(suggestion => {
      const firstChar = suggestion.label.charAt(0).toLowerCase();
      if (!index.has(firstChar)) {
        index.set(firstChar, []);
      }
      index.get(firstChar)!.push(suggestion);
    });
    
    return index;
  }, []);

  // Get current word being typed and cursor position
  const getCurrentWordAndPosition = useCallback(() => {
    if (!textareaRef.current) return { word: "", start: 0, end: 0 };

    const textarea = textareaRef.current;
    const cursorPos = textarea.selectionStart;
    const text = textarea.value;

    // Find word boundaries
    let start = cursorPos;
    let end = cursorPos;

    // Move backwards to find start of word (including :: for namespaces)
    while (start > 0 && /[a-zA-Z0-9_:.]/.test(text[start - 1])) {
      start--;
    }

    // Move forwards to find end of word
    while (end < text.length && /[a-zA-Z0-9_]/.test(text[end])) {
      end++;
    }

    const word = text.substring(start, end);
    return { word, start, end };
  }, [textareaRef]);

  // Filter suggestions based on current input with indexed lookup for performance
  const getFilteredSuggestions = useCallback(
    (word: string): AutocompleteSuggestion[] => {
      if (word.length < 1) return [];

      const lowerWord = word.toLowerCase();
      const firstChar = lowerWord.charAt(0);
      
      // Use indexed lookup for better performance with large suggestion sets
      const candidateSuggestions = indexedSuggestions.get(firstChar) || [];
      
      const filtered = candidateSuggestions.filter((suggestion) =>
        suggestion.label.toLowerCase().startsWith(lowerWord),
      );

      // Sort by relevance: exact matches first, then by type priority
      return filtered.sort((a, b) => {
        const aExact = a.label.toLowerCase() === lowerWord ? 0 : 1;
        const bExact = b.label.toLowerCase() === lowerWord ? 0 : 1;

        if (aExact !== bExact) return aExact - bExact;

        // Type priority: keywords, then functions, then variables, then operators
        const typePriority: Record<string, number> = {
          keyword: 0,
          function: 1,
          variable: 2,
          namespace: 3,
          operator: 4,
        };

        return (typePriority[a.type] || 5) - (typePriority[b.type] || 5);
      });
    },
    [indexedSuggestions],
  );

  // Calculate autocomplete dropdown position
  const calculatePosition = useCallback(() => {
    if (!textareaRef.current) return;

    const textarea = textareaRef.current;
    const { start } = getCurrentWordAndPosition();

    // Create a temporary element to measure text width
    const temp = document.createElement("div");
    temp.style.position = "absolute";
    temp.style.visibility = "hidden";
    temp.style.whiteSpace = "pre-wrap";
    temp.style.font = window.getComputedStyle(textarea).font;
    temp.textContent = textarea.value.substring(0, start);

    document.body.appendChild(temp);
    const textWidth = temp.offsetWidth;
    document.body.removeChild(temp);

    const textareaRect = textarea.getBoundingClientRect();
    const lines = textarea.value.substring(0, start).split("\n");
    const currentLineText = lines[lines.length - 1];

    // Approximate character width
    const charWidth = textWidth / (currentLineText.length || 1);
    const lineHeight =
      parseInt(window.getComputedStyle(textarea).lineHeight) || 20;

    setPosition({
      top: textareaRect.top + (lines.length - 1) * lineHeight + lineHeight,
      left: textareaRect.left + currentLineText.length * charWidth,
    });
  }, [textareaRef, getCurrentWordAndPosition]);

  // Update suggestions when text changes
  useEffect(() => {
    if (!visible) return;

    const { word } = getCurrentWordAndPosition();
    const filtered = getFilteredSuggestions(word);

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
    calculatePosition,
    onVisibilityChange,
  ]);

  // Handle keyboard navigation
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

  // Click outside to close
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
      case "namespace":
        return "text-teal-600 dark:text-teal-400";
      case "variable":
        return "text-green-600 dark:text-green-400";
      case "operator":
        return "text-red-600 dark:text-red-400";
      default:
        return "text-gray-600 dark:text-gray-400";
    }
  };

  return (
    <div
      ref={containerRef}
      className="z-50 mt-2 mr-2 max-h-64 w-90 overflow-auto rounded-lg border bg-popover/70 shadow-lg"
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
            <div className="mt-1 font-mono text-xs text-muted-foreground">
              {suggestion.detail}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

export default RhaiAutocomplete;
