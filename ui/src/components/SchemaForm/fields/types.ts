import type { FieldNode, FieldPath } from "@flow/lib/schemaForm";

/**
 * What every field component receives.
 *
 * `node` is the compiled schema and `path` is where this instance of it lives
 * in the data — the two diverge for array items, where one item node describes
 * every index, and they are deliberately separate so nothing has to guess.
 */
export type FieldProps<N extends FieldNode = FieldNode> = {
  node: N;
  path: FieldPath;
  value: unknown;
  required?: boolean;
  /** Rendered by a parent that already showed the label (array items). */
  hideLabel?: boolean;
  onChange: (path: FieldPath, value: unknown) => void;
};
