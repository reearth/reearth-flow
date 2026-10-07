import { XIcon } from "@phosphor-icons/react";
import { useCallback, useMemo, useState } from "react";

import {
  Input,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@flow/components";
import { IconButton } from "@flow/components/buttons";
import { useT } from "@flow/lib/i18n";
import type { ColorField as ColorFieldNode } from "@flow/lib/schemaForm";

import ActionArea from "../components/ActionArea";

import { FieldRow } from "./FieldRow";
import type { FieldProps } from "./types";
import { useField } from "./useField";

// Offered while the field is unset. The native input always holds some colour,
// so it cannot stand in for "none": picking the colour it already holds would
// fire no change. Each swatch here is an explicit write instead.
const PALETTE = [
  "#000000",
  "#444444",
  "#888888",
  "#bbbbbb",
  "#ffffff",
  "#e60000",
  "#ff9900",
  "#ffff00",
  "#008a00",
  "#0066cc",
  "#9933ff",
  "#f06666",
  "#ffc266",
  "#66b966",
  "#66a3e0",
  "#c285ff",
];

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
  const [paletteOpen, setPaletteOpen] = useState(false);

  const handleReset = useCallback(
    () => onChange(path, initialValue),
    [onChange, path, initialValue],
  );

  const handlePick = useCallback(
    (color: string) => {
      onChange(path, color);
      setPaletteOpen(false);
    },
    [onChange, path],
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
        {isUnset ? (
          <Popover open={paletteOpen} onOpenChange={setPaletteOpen}>
            <PopoverTrigger
              id={field.id}
              disabled={field.readonly}
              aria-label={t("Choose a color")}
              aria-required={required}
              aria-invalid={field.hasErrors}
              data-unset
              onFocus={field.onFocus}
              onBlur={field.onBlur}
              className={`${field.hasErrors ? "border-destructive" : ""} flex h-7 w-20 cursor-pointer items-stretch rounded-md border px-[2px] py-[4px] shadow-sm disabled:cursor-not-allowed disabled:opacity-50`}
              style={field.awarenessStyle}>
              <span className="flex-1 border border-[#777] bg-[linear-gradient(to_top_right,transparent_calc(50%-1px),var(--destructive),transparent_calc(50%+1px))]" />
            </PopoverTrigger>
            <PopoverContent
              align="start"
              className="grid w-auto grid-cols-8 gap-1 p-2">
              {PALETTE.map((color) => (
                <button
                  key={color}
                  type="button"
                  aria-label={color}
                  className="h-5 w-5 cursor-pointer rounded-sm border border-[#777] hover:scale-110"
                  style={{ backgroundColor: color }}
                  onClick={() => handlePick(color)}
                />
              ))}
            </PopoverContent>
          </Popover>
        ) : (
          <Input
            id={field.id}
            name={field.id}
            type="color"
            disabled={field.readonly}
            required={required}
            value={value}
            onChange={(event) => onChange(path, event.target.value)}
            onFocus={field.onFocus}
            onBlur={field.onBlur}
            aria-required={required}
            aria-invalid={field.hasErrors}
            className={`${field.hasErrors ? "border-destructive" : ""} h-7 w-20 p-0`}
            style={field.awarenessStyle}
          />
        )}
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
