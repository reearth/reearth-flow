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
  const showNone = isUnset && canClear;

  const handleReset = useCallback(
    () => onChange(path, initialValue),
    [onChange, path, initialValue],
  );

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
          value={isUnset ? (initialValue ?? "#000000") : value}
          onChange={(event) => onChange(path, event.target.value)}
          onFocus={field.onFocus}
          onBlur={field.onBlur}
          aria-required={required}
          aria-invalid={field.hasErrors}
          className={`${field.hasErrors ? "border-destructive" : ""} ${showNone ? NONE_SWATCH : ""} h-7 w-20 p-0`}
          data-unset={showNone || undefined}
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
