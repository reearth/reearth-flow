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
  // A required field left blank is marked by the asterisk beside its label and
  // nothing else: no sentence underneath and no red border. It still counts
  // against the form's validity — that is what gates Update — but a field the
  // user simply has not reached yet is not something to colour in.
  //
  // Anything with a message to show does get the border, since there the colour
  // points at something the field itself cannot say.
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
    hasErrors: messages !== undefined,
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
