import { Checkbox } from "@flow/components";
import type { BooleanField as BooleanFieldNode } from "@flow/lib/schemaForm";

import { FieldRow } from "./FieldRow";
import type { FieldProps } from "./types";
import { useField } from "./useField";

const BooleanField: React.FC<FieldProps<BooleanFieldNode>> = ({
  node,
  path,
  value,
  required,
  onChange,
}) => {
  const field = useField(path);
  const label = node.title ?? node.name;

  return (
    <FieldRow
      id={field.id}
      label={label}
      required={required}
      errors={field.errors}
      hideLabel>
      <div className="flex items-center gap-2 py-2">
        <Checkbox
          id={field.id}
          name={field.id}
          style={field.awarenessStyle}
          checked={
            value === undefined || value === null ? false : Boolean(value)
          }
          disabled={field.readonly}
          onCheckedChange={(checked) => onChange(path, checked)}
          onFocus={field.onFocus}
          onBlur={field.onBlur}
          aria-describedby={field.describedBy}
        />
        <label className="text-xs" htmlFor={field.id}>
          {label}
        </label>
      </div>
    </FieldRow>
  );
};

export { BooleanField };
