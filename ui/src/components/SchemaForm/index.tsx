import type { JSONSchema7 } from "json-schema";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { extractDescriptions } from "@flow/features/Editor/components/ParamsDialog/utils/extractDescriptions";
import { useT } from "@flow/lib/i18n";
import {
  applyDefaults,
  compile,
  deleteAtPath,
  isValid,
  setAtPath,
  pathKey,
  validate,
  type FieldPath,
  type ObjectField as ObjectFieldNode,
} from "@flow/lib/schemaForm";
import type { AwarenessUser } from "@flow/types";

import { SchemaFormErrorBoundary } from "./components/SchemaFormErrorBoundary";
import { SchemaFormProvider, type EditorContext } from "./context";
import { Field } from "./fields/Field";
import { ObjectField } from "./fields/ObjectField";

export type { EditorContext } from "./context";
export type { CodeValue } from "./fields/ExprField";

type SchemaFormProps = {
  readonly?: boolean;
  /** The action's schema, exactly as the engine published it. */
  schema?: JSONSchema7;
  actionName?: string;
  defaultFormData?: unknown;
  /** Other users' focus, keyed by the field's dot path. */
  fieldFocusMap?: Record<string, AwarenessUser[]>;
  onFieldFocus?: (fieldKey: string | null) => void;
  onChange: (data: unknown, changedFieldKey?: string) => void;
  onValidationChange?: (isValid: boolean) => void;
  /**
   * The generic value editor. Currently unreachable: it was only ever offered
   * on a field typed `$ref: "#/definitions/Expr"`, which the engine no longer
   * emits — every expression field is now `format: "code"` and opens the
   * FlowExpr or Python editor instead. Kept plumbed for when a field needs it.
   */
  onEditorOpen?: (context: EditorContext) => void;
  onPythonEditorOpen?: (context: EditorContext) => void;
  onFlowExprEditorOpen?: (context: EditorContext) => void;
};

/**
 * The action parameter form.
 *
 * The schema is compiled once into a field tree (`@flow/lib/schemaForm`) and
 * rendered from that. Validation runs against the schema as published, so what
 * the form calls valid is what the engine calls valid — see
 * `docs/schema-form-replacement.md` for what that replaced and why.
 */
const SchemaForm: React.FC<SchemaFormProps> = ({
  readonly,
  schema,
  actionName,
  defaultFormData,
  fieldFocusMap,
  onFieldFocus,
  onChange,
  onValidationChange,
  onEditorOpen,
  onPythonEditorOpen,
  onFlowExprEditorOpen,
}) => {
  const t = useT();

  const root = useMemo(
    () => (schema ? compile(schema, { actionName }) : undefined),
    [schema, actionName],
  );

  const descriptions = useMemo(
    () => (schema ? extractDescriptions(schema) : {}),
    [schema],
  );

  // Seeding is display-only: the defaults the schema states are shown, but
  // nothing is written back until the user changes something. Mounting a
  // dialog is not an edit.
  const seeded = useMemo(
    () => (root ? applyDefaults(root, defaultFormData) : defaultFormData),
    [root, defaultFormData],
  );

  const errors = useMemo(
    // Only an absent root becomes `{}`; a root the schema legally allows to be
    // `null` must reach AJV as `null`, or a valid form is reported invalid.
    () => (schema ? validate(schema, seeded === undefined ? {} : seeded) : {}),
    [schema, seeded],
  );
  const valid = isValid(errors);

  // Errors are held back until the user engages, so opening a half-filled
  // action does not greet them with a wall of red they did not cause.
  const [touched, setTouched] = useState(false);
  useEffect(() => setTouched(false), [schema]);

  const onValidationChangeRef = useRef(onValidationChange);
  onValidationChangeRef.current = onValidationChange;
  useEffect(() => {
    onValidationChangeRef.current?.(valid);
  }, [valid]);

  const handleChange = useCallback(
    (path: FieldPath, value: unknown) => {
      setTouched(true);
      // Clearing a field removes its key. Writing `undefined` would leave the
      // key present — `Object.keys` still reports it, so an object with
      // `additionalProperties: false` rejects it — and it would ride into the
      // saved params as a phantom entry.
      const next =
        value === undefined
          ? deleteAtPath(seeded, path)
          : setAtPath(seeded, path, value);
      onChange(next, pathKey(path));
    },
    [onChange, seeded],
  );

  const context = useMemo(
    () => ({
      errors,
      showErrors: touched,
      readonly,
      fieldFocusMap,
      onFieldFocus,
      onEditorOpen,
      onPythonEditorOpen,
      onFlowExprEditorOpen,
    }),
    [
      errors,
      touched,
      readonly,
      fieldFocusMap,
      onFieldFocus,
      onEditorOpen,
      onPythonEditorOpen,
      onFlowExprEditorOpen,
    ],
  );

  if (!root) {
    return schema ? (
      <p className="text-destructive">{t("Error with the schema")}</p>
    ) : null;
  }

  return (
    <SchemaFormErrorBoundary>
      <SchemaFormProvider value={context}>
        <div className="flex-1 overflow-scroll">
          {root.kind === "object" ? (
            <ObjectField
              node={root as ObjectFieldNode}
              path={[]}
              value={seeded}
              isRoot
              descriptions={descriptions}
              onChange={handleChange}
            />
          ) : (
            <Field
              node={root}
              path={[]}
              value={seeded}
              onChange={handleChange}
            />
          )}
        </div>
      </SchemaFormProvider>
    </SchemaFormErrorBoundary>
  );
};

export { SchemaForm };
