import { CaretDownIcon } from "@phosphor-icons/react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { EnumField as EnumFieldNode } from "@flow/lib/schemaForm";

import { FieldRow } from "./FieldRow";
import type { FieldProps } from "./types";
import { useField } from "./useField";

const EnumField: React.FC<FieldProps<EnumFieldNode>> = ({
  node,
  path,
  value,
  required,
  hideLabel,
  onChange,
}) => {
  const t = useT();
  const field = useField(path);

  const selected = node.options.find((option) => option.value === value);
  // An enum that may be unset needs a way back to unset. The old form offered
  // a "-" that produced `undefined` against a schema which no longer permitted
  // absence at all, so the choice was both unreachable and invalid.
  const canClear = node.nullable && !required;

  return (
    <FieldRow
      id={field.id}
      label={node.title ?? node.name}
      required={required}
      errors={field.errors}
      hideLabel={hideLabel}>
      <DropdownMenu modal>
        <DropdownMenuTrigger
          id={field.id}
          className={`flex h-8 max-w-141 min-w-[30%] items-center justify-between gap-2 rounded border bg-background px-3 hover:bg-accent ${
            field.hasErrors ? "border-destructive" : ""
          }`}
          style={field.awarenessStyle}
          disabled={field.readonly}
          onFocus={field.onFocus}
          onBlur={field.onBlur}
          aria-label={node.title ?? node.name}
          aria-required={required}
          aria-invalid={field.hasErrors}
          aria-describedby={field.describedBy}>
          <span className={selected ? "" : "text-muted-foreground"}>
            {selected?.label ?? t("Not set")}
          </span>
          <CaretDownIcon className="size-4" />
        </DropdownMenuTrigger>
        <DropdownMenuContent className="max-h-60 overflow-auto" align="start">
          {canClear && (
            <DropdownMenuItem
              className={`text-muted-foreground ${value == null ? "bg-accent" : ""}`}
              onClick={() => onChange(path, undefined)}>
              {t("Not set")}
            </DropdownMenuItem>
          )}
          {node.options.map((option) => (
            <DropdownMenuItem
              key={String(option.value)}
              className={value === option.value ? "bg-accent" : ""}
              onClick={() => onChange(path, option.value)}>
              {option.label}
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
    </FieldRow>
  );
};

export { EnumField };
