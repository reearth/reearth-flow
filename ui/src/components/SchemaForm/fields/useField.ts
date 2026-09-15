import { useCallback, useMemo } from "react";

import { pathKey, type FieldPath } from "@flow/lib/schemaForm";

import { useSchemaForm } from "../context";
import { paramsAwarenessStyles } from "../utils/awarenessTemplateStyles";

/**
 * The per-field slice of form context: its key, its errors, who else is
 * looking at it, and the focus handlers that broadcast the same.
 */
export const useField = (path: FieldPath) => {
  const {
    errors,
    showErrors,
    readonly,
    fieldFocusMap,
    onFieldFocus,
    onEditorOpen,
    onPythonEditorOpen,
    onFlowExprEditorOpen,
  } = useSchemaForm();

  const key = useMemo(() => pathKey(path), [path]);
  // A field can be invalid with nothing to say — a required field left blank is
  // marked by its asterisk and its red border, so `hasErrors` (the highlight)
  // and `errors` (the sentence beneath) are not the same question.
  const entry = showErrors ? errors[key] : undefined;
  const messages = entry && entry.length > 0 ? entry : undefined;
  const focusedUsers = fieldFocusMap?.[key];

  const handleFocus = useCallback(
    () => onFieldFocus?.(key),
    [onFieldFocus, key],
  );
  const handleBlur = useCallback(() => onFieldFocus?.(null), [onFieldFocus]);

  const id = `field-${key.replace(/\./g, "-") || "root"}`;

  return {
    key,
    // A DOM id has to be a valid selector target, which a dot path is not.
    id,
    errors: messages,
    hasErrors: entry !== undefined,
    describedBy: messages ? `${id}-error` : undefined,
    readonly,
    awarenessStyle: paramsAwarenessStyles(focusedUsers),
    onFocus: handleFocus,
    onBlur: handleBlur,
    onEditorOpen,
    onPythonEditorOpen,
    onFlowExprEditorOpen,
  };
};
