import { useCallback, useRef, useState } from "react";

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

  /**
   * Whether the input is holding something that is not yet a number.
   *
   * A number on its way to being typed passes through states that are not
   * numbers — `-`, `1.`, `1e`. A `type="number"` input reports these as an
   * empty `value` with `validity.badInput` set, keeping the characters on
   * screen in a buffer of its own that nothing can read.
   *
   * The way to keep them there is to leave the control alone: while the entry
   * is unparseable this renders `""`, which is what the input already reports,
   * so React finds nothing to write and the half-typed number survives. Writing
   * anything else — the stored value, or a draft of the raw text — resets the
   * control and takes the characters with it, which is why a lone minus
   * disappeared before the digits could follow it.
   */
  const [isPartial, setIsPartial] = useState(false);

  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const partial = event.target.validity?.badInput ?? false;
      setIsPartial(partial);
      // Nothing is stored for a half-typed number, and the value it is
      // replacing is left where it is until the user finishes.
      if (partial) return;

      const raw = event.target.value;
      if (raw === "") {
        onChange(path, undefined);
        return;
      }

      const parsed = node.integer ? parseInt(raw, 10) : parseFloat(raw);
      if (!Number.isNaN(parsed)) onChange(path, parsed);
    },
    [onChange, path, node.integer],
  );

  const handleBlur = useCallback(() => {
    // Editing is over: drop a half-typed entry and show what was stored.
    setIsPartial(false);
    field.onBlur();
  }, [field]);

  const handleReset = useCallback(() => {
    setIsPartial(false);
    onChange(path, defaultValue.current);
  }, [onChange, path]);

  const display = isPartial || typeof value !== "number" ? "" : String(value);

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
          value={display}
          onChange={handleChange}
          onFocus={field.onFocus}
          onBlur={handleBlur}
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
