import { useCallback, useRef } from "react";

import { Input } from "@flow/components";
import type { NumberField as NumberFieldNode } from "@flow/lib/schemaForm";

import ActionArea from "../components/ActionArea";

import { FieldRow } from "./FieldRow";
import type { FieldProps } from "./types";
import { useField } from "./useField";

const NumberField: React.FC<FieldProps<NumberFieldNode>> = ({
  node,
  path,
  value,
  required,
  hideLabel,
  onChange,
}) => {
  const field = useField(path);
  const defaultValue = useRef(value ?? node.default ?? "");

  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const raw = event.target.value;
      if (raw === "") {
        onChange(path, undefined);
        return;
      }
      const parsed = node.integer ? parseInt(raw, 10) : parseFloat(raw);
      // A half-typed number ("-", "1e") parses to NaN; holding the previous
      // value keeps the keystroke from being swallowed.
      if (!Number.isNaN(parsed)) onChange(path, parsed);
    },
    [onChange, path, node.integer],
  );

  const handleReset = useCallback(
    () => onChange(path, defaultValue.current),
    [onChange, path],
  );

  return (
    <FieldRow
      id={field.id}
      label={node.title ?? node.name}
      required={required}
      errors={field.errors}
      hideLabel={hideLabel}>
      <div className="flex items-center gap-2">
        <Input
          id={field.id}
          name={field.id}
          type="number"
          disabled={field.readonly}
          required={required}
          value={typeof value === "number" ? value : ""}
          onChange={handleChange}
          onFocus={field.onFocus}
          onBlur={field.onBlur}
          min={node.minimum}
          max={node.maximum}
          step={node.integer ? 1 : "any"}
          aria-required={required}
          aria-invalid={field.hasErrors}
          aria-describedby={field.describedBy}
          className={field.hasErrors ? "border-destructive" : ""}
          style={field.awarenessStyle}
        />
        <ActionArea
          value={value}
          defaultValue={defaultValue}
          readonly={field.readonly}
          onReset={handleReset}
        />
      </div>
    </FieldRow>
  );
};

export { NumberField };
