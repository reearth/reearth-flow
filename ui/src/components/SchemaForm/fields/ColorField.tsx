import { useCallback, useRef } from "react";

import { Input } from "@flow/components";
import type { ColorField as ColorFieldNode } from "@flow/lib/schemaForm";

import ActionArea from "../components/ActionArea";

import { FieldRow } from "./FieldRow";
import type { FieldProps } from "./types";
import { useField } from "./useField";

const ColorField: React.FC<FieldProps<ColorFieldNode>> = ({
  node,
  path,
  value,
  required,
  hideLabel,
  onChange,
}) => {
  const field = useField(path);
  const defaultValue = useRef(
    (value as string) ?? (node.default as string) ?? "#000000",
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
      <div className="flex w-full items-center justify-between gap-2">
        <Input
          id={field.id}
          name={field.id}
          type="color"
          disabled={field.readonly}
          required={required}
          value={typeof value === "string" ? value : defaultValue.current}
          onChange={(event) => onChange(path, event.target.value)}
          onFocus={field.onFocus}
          onBlur={field.onBlur}
          aria-required={required}
          aria-invalid={field.hasErrors}
          className={`${field.hasErrors ? "border-destructive" : ""} h-7 w-20 p-0`}
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

export { ColorField };
