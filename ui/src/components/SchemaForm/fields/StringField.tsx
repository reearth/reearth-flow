import { useCallback, useRef } from "react";

import { Input } from "@flow/components";
import type { StringField as StringFieldNode } from "@flow/lib/schemaForm";

import ActionArea from "../components/ActionArea";

import { FieldRow } from "./FieldRow";
import type { FieldProps } from "./types";
import { useField } from "./useField";

const StringField: React.FC<FieldProps<StringFieldNode>> = ({
  node,
  path,
  value,
  required,
  hideLabel,
  onChange,
}) => {
  const field = useField(path);
  const defaultValue = useRef(
    (value as string) ?? (node.default as string) ?? "",
  );

  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const next = event.target.value;
      // An emptied optional field is unset, not an empty string — the engine
      // reads a missing key and a "" differently.
      onChange(path, next === "" && node.nullable ? undefined : next);
    },
    [onChange, path, node.nullable],
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
      <div className="flex w-full items-center gap-2">
        <Input
          id={field.id}
          name={field.id}
          type="text"
          disabled={field.readonly}
          required={required}
          value={typeof value === "string" ? value : ""}
          onChange={handleChange}
          onFocus={field.onFocus}
          onBlur={field.onBlur}
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

export { StringField };
