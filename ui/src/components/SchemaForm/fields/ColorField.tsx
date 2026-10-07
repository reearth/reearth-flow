import { XIcon } from "@phosphor-icons/react";
import { useCallback, useMemo, useState } from "react";

import { Input } from "@flow/components";
import { IconButton } from "@flow/components/buttons";
import { useT } from "@flow/lib/i18n";
import type { ColorField as ColorFieldNode } from "@flow/lib/schemaForm";

import ActionArea from "../components/ActionArea";

import { FieldRow } from "./FieldRow";
import type { FieldProps } from "./types";
import { useField } from "./useField";

// The native input always holds some colour, so while the field is unset it
// holds this one under a "none" swatch. Opening the picker stores it first:
// choosing the colour the input already holds fires no change, so without that
// write, picking it would leave the field unset.
const UNSET_SEED = "#000000";

const NONE_SWATCH =
  "[&::-moz-color-swatch]:bg-transparent! [&::-moz-color-swatch]:bg-[linear-gradient(to_top_right,transparent_calc(50%-1px),var(--destructive),transparent_calc(50%+1px))] [&::-webkit-color-swatch]:bg-transparent! [&::-webkit-color-swatch]:bg-[linear-gradient(to_top_right,transparent_calc(50%-1px),var(--destructive),transparent_calc(50%+1px))]";

const ColorField: React.FC<FieldProps<ColorFieldNode>> = ({
  node,
  path,
  value,
  required,
  hideLabel,
  onChange,
}) => {
  const t = useT();
  const field = useField(path);
  const [initialValue] = useState(
    () => (value as string | undefined) ?? (node.default as string | undefined),
  );
  const defaultValue = useMemo(
    () => ({ current: initialValue }),
    [initialValue],
  );
  const isUnset = typeof value !== "string" || !value;
  const canClear = !required && node.default === undefined;

  const handleReset = useCallback(
    () => onChange(path, initialValue),
    [onChange, path, initialValue],
  );

  // Runs before the browser opens the picker.
  const handleOpen = useCallback(() => {
    if (isUnset) onChange(path, UNSET_SEED);
  }, [isUnset, onChange, path]);

  const handleClear = useCallback(
    () => onChange(path, undefined),
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
          value={isUnset ? UNSET_SEED : value}
          onClick={handleOpen}
          onChange={(event) => onChange(path, event.target.value)}
          onFocus={field.onFocus}
          onBlur={field.onBlur}
          aria-required={required}
          aria-invalid={field.hasErrors}
          data-unset={isUnset || undefined}
          className={`${field.hasErrors ? "border-destructive" : ""} ${isUnset ? NONE_SWATCH : ""} h-7 w-20 p-0`}
          style={field.awarenessStyle}
        />
        <div className="flex items-center">
          {canClear && (
            <IconButton
              icon={<XIcon />}
              disabled={isUnset || field.readonly}
              tooltipText={t("Clear")}
              aria-label={t("Clear")}
              onClick={handleClear}
            />
          )}
          <ActionArea
            value={value}
            defaultValue={defaultValue}
            readonly={field.readonly}
            onReset={
              canClear && initialValue === undefined ? undefined : handleReset
            }
          />
        </div>
      </div>
    </FieldRow>
  );
};

export { ColorField };
