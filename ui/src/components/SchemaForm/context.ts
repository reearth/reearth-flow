/**
 * What every field needs but none of them own.
 *
 * Passed as one object rather than through a framework registry, so a field
 * component's dependencies are visible in its props instead of being reached
 * for at render time.
 */
import { createContext, useContext } from "react";

import type { FieldPath, ValidationErrors } from "@flow/lib/schemaForm";
import type { AwarenessUser } from "@flow/types";

export type EditorContext = {
  /** Dot path of the field, e.g. `response.responseEncoding`. */
  key: string;
  name: string;
  path: FieldPath;

  value: any;
  /**
   * The field's own schema fragment. Loosely typed because the editor dialogs
   * read whatever they need off it (`title`, `type`, `properties.type.enum`).
   */

  schema: any;
  fieldName: string;
};

export type SchemaFormContextValue = {
  errors: ValidationErrors;
  /** Suppresses errors until the user has engaged with the form. */
  showErrors: boolean;
  readonly?: boolean;
  fieldFocusMap?: Record<string, AwarenessUser[]>;
  onFieldFocus?: (key: string | null) => void;
  onEditorOpen?: (context: EditorContext) => void;
  onPythonEditorOpen?: (context: EditorContext) => void;
  onFlowExprEditorOpen?: (context: EditorContext) => void;
};

const SchemaFormContext = createContext<SchemaFormContextValue>({
  errors: {},
  showErrors: false,
});

export const SchemaFormProvider = SchemaFormContext.Provider;

export const useSchemaForm = (): SchemaFormContextValue =>
  useContext(SchemaFormContext);
