import type { ReactNode } from "react";

/**
 * Label, control and errors for one field — the layout every leaf shares.
 *
 * `FieldRow` is given the rendered control rather than deciding what it is, so
 * the arrangement lives in one place while each field kind stays responsible
 * only for its own input.
 *
 * A leaf's `description` is deliberately not rendered here. Repeating a
 * sentence under every input triples the height of a form like Feature Filter's
 * condition list; descriptions belong to sections, and to the tooltip on the
 * form's own title. Where an input can carry one as placeholder text, it does.
 */
type Props = {
  id: string;
  label?: string;
  required?: boolean;
  errors?: string[];
  /** Checkboxes and rich-text editors carry their own label. */
  hideLabel?: boolean;
  children: ReactNode;
};

const FieldRow: React.FC<Props> = ({
  id,
  label,
  required,
  errors,
  hideLabel,
  children,
}) => (
  <div className="my-1.5">
    {hideLabel || !label ? (
      children
    ) : (
      <div className="flex flex-1 items-center gap-6">
        <div className="flex flex-row gap-1">
          {/* A real label/control association, so the field is reachable by its
              name to a screen reader as well as to a test. */}
          <label className="shrink-0 font-light" htmlFor={id}>
            {label}
          </label>
          {required && <p className="h-2 font-thin text-destructive">*</p>}
        </div>
        <div className="min-w-0 flex-1">{children}</div>
      </div>
    )}
    {errors && errors.length > 0 && (
      <div
        id={`${id}-error`}
        className="mt-1 text-xs text-destructive"
        role="alert">
        {errors.join(". ")}
      </div>
    )}
  </div>
);

export { FieldRow };
