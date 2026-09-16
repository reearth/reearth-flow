import { CaretDownIcon } from "@phosphor-icons/react";
import { useCallback, useRef, useState } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import {
  applyDefaults,
  selectVariant,
  type UnionField as UnionFieldNode,
  type UnionVariant,
} from "@flow/lib/schemaForm";

import { Field } from "./Field";
import { FieldRow } from "./FieldRow";
import type { FieldProps } from "./types";
import { useField } from "./useField";

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

/** Property names a variant can hold, used to decide what survives a switch. */
const keysOf = (variant: UnionVariant): string[] =>
  variant.node.kind === "object"
    ? variant.node.properties.map((property) => property.name)
    : [];

const UnionField: React.FC<FieldProps<UnionFieldNode>> = ({
  node,
  path,
  value,
  required,
  hideLabel,
  onChange,
}) => {
  const t = useT();
  const field = useField(path);

  /**
   * The variant the user picked, when the value cannot say so itself.
   *
   * An untagged variant is recognised by the fields it carries, so the moment
   * it is chosen — before any of them are filled in — the value matches
   * nothing. Choosing "CRS" wrote `{}`, which identified no variant, and the
   * control fell straight back to showing nothing selected.
   *
   * A value that does identify a variant always wins, so a collaborator
   * switching it is still followed.
   */
  const [chosen, setChosen] = useState<number | null>(null);
  const matched = selectVariant(node, value);
  const index = matched >= 0 ? matched : (chosen ?? -1);
  const selected = index >= 0 ? node.variants[index] : undefined;
  const title = node.title ?? node.name;

  // What each variant held last time it was selected, so switching away and
  // back is not destructive. RJSF discarded it.
  const stash = useRef<Record<string, Record<string, unknown>>>({});

  const handleSelect = useCallback(
    (nextIndex: number) => {
      const variant = node.variants[nextIndex];

      if (selected && isRecord(value)) {
        stash.current[selected.key] = value;
      }

      setChosen(nextIndex);

      if (variant.constant !== undefined) {
        onChange(path, variant.constant);
        return;
      }

      if (variant.node.kind !== "object") {
        // A scalar variant is told apart by the type of the value itself, so
        // there is nothing to write until the user types one. Writing `{}` —
        // which is what fell out of seeding an object into a string — stored an
        // object where the schema wants a string, and made all three of Text,
        // Number and True-or-False identical.
        onChange(path, undefined);
        return;
      }

      // Keys the target variant also has carry over — switching a CSV reader
      // between WKT and coordinate columns should not clear the column names
      // the two spellings share.
      const target = new Set(keysOf(variant));
      const carried = isRecord(value)
        ? Object.fromEntries(
            Object.entries(value).filter(([key]) => target.has(key)),
          )
        : {};

      const seeded = applyDefaults(variant.node, {
        ...(stash.current[variant.key] ?? {}),
        ...carried,
      });
      const next = isRecord(seeded) ? { ...seeded } : {};

      if (variant.discriminator) {
        next[variant.discriminator.property] = variant.discriminator.value;
      } else {
        // An untagged variant is recognised by the keys it carries, so a choice
        // with none of them filled in is invisible — including to the other
        // people editing the node, who see only the `{}` it used to write and
        // no reason to move their own control. Marking the required keys as
        // present-but-empty puts the choice in the data, where it travels.
        for (const key of variant.requiredKeys) {
          if (next[key] === undefined) next[key] = null;
        }
      }

      onChange(path, next);
    },
    [node.variants, onChange, path, selected, value],
  );

  const canClear = node.nullable && !required;
  // "Not set" is a state this field can be left in; where it cannot be, the
  // placeholder is a prompt instead. `destinationFrame` is required with no
  // null branch, so nothing there is unset — it is only unchosen.
  const placeholder = canClear ? t("Not set") : t("Select...");

  return (
    <div>
      <FieldRow
        id={field.id}
        label={title}
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
            aria-label={title}
            aria-required={required}
            aria-invalid={field.hasErrors}>
            <span className={selected ? "" : "text-muted-foreground"}>
              {selected?.title ?? placeholder}
            </span>
            <CaretDownIcon className="size-4" />
          </DropdownMenuTrigger>
          <DropdownMenuContent className="max-h-60 overflow-auto" align="start">
            {canClear && (
              <DropdownMenuItem
                className={`text-muted-foreground ${selected ? "" : "bg-accent"}`}
                onClick={() => {
                  setChosen(null);
                  onChange(path, undefined);
                }}>
                {t("Not set")}
              </DropdownMenuItem>
            )}
            {node.variants.map((variant, variantIndex) => (
              <DropdownMenuItem
                key={variant.key}
                className={variantIndex === index ? "bg-accent" : ""}
                onClick={() => handleSelect(variantIndex)}>
                {variant.title}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </FieldRow>
      {selected && selected.constant === undefined && (
        <div className="pl-2">
          {selected.description && (
            <div className="mt-1 text-xs text-muted-foreground">
              {selected.description}
            </div>
          )}
          {/* The variant's own title is the dropdown's label, so the section
              heading underneath it would only repeat the choice back. */}
          <Field
            node={selected.node}
            path={path}
            value={value}
            hideLabel
            onChange={onChange}
          />
        </div>
      )}
    </div>
  );
};

export { UnionField };
