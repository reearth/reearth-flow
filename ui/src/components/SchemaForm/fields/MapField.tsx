import { PlusIcon, TrashIcon } from "@phosphor-icons/react";
import { useCallback, useMemo, useState } from "react";

import { Button, Input } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { childPath } from "@flow/lib/schemaForm";
import type { MapField as MapFieldNode } from "@flow/lib/schemaForm";

import { Field } from "./Field";
import { SectionHeading } from "./SectionHeading";
import type { FieldProps } from "./types";
import { useField } from "./useField";

/**
 * `additionalProperties: <schema>` — a set of entries the user names.
 *
 * Keys are edited on blur rather than per keystroke, so renaming a key does
 * not rewrite the object on every character and lose focus mid-word.
 */
const NEW_KEY = "newKey";

const MapField: React.FC<FieldProps<MapFieldNode>> = ({
  node,
  path,
  value,
  required,
  onChange,
}) => {
  const t = useT();
  const field = useField(path);
  const [draftKeys, setDraftKeys] = useState<Record<string, string>>({});

  const record = useMemo(
    () => (value ?? {}) as Record<string, unknown>,
    [value],
  );
  const entries = Object.entries(record);

  const handleRename = useCallback(
    (from: string, to: string) => {
      setDraftKeys((drafts) => {
        const { [from]: _removed, ...rest } = drafts;
        return rest;
      });
      if (to === from || to === "") return;
      if (Object.prototype.hasOwnProperty.call(record, to)) return;
      onChange(
        path,
        Object.fromEntries(
          Object.entries(record).map(([key, entry]) =>
            key === from ? [to, entry] : [key, entry],
          ),
        ),
      );
    },
    [onChange, path, record],
  );

  const handleRemove = useCallback(
    (key: string) =>
      onChange(
        path,
        Object.fromEntries(
          Object.entries(record).filter(([entry]) => entry !== key),
        ),
      ),
    [onChange, path, record],
  );

  const handleAdd = useCallback(() => {
    // Not translated: this becomes a key in the params the engine reads, so a
    // localised one would mean a French user producing different data.
    let name = NEW_KEY;
    let suffix = 1;
    while (Object.prototype.hasOwnProperty.call(record, name)) {
      name = `${NEW_KEY}${++suffix}`;
    }
    onChange(path, { ...record, [name]: undefined });
  }, [onChange, path, record]);

  return (
    <div>
      <SectionHeading
        id={`${field.id}-title`}
        title={node.title ?? node.name}
        required={required}
      />
      {node.description && (
        <div className="mt-1 shrink-0 text-xs text-muted-foreground">
          {node.description}
        </div>
      )}
      {entries.map(([key, entry]) => (
        <div key={key} className="flex items-start gap-2 py-1">
          <Input
            className="w-40 shrink-0"
            value={draftKeys[key] ?? key}
            disabled={field.readonly}
            onChange={(event) =>
              setDraftKeys((drafts) => ({
                ...drafts,
                [key]: event.target.value,
              }))
            }
            onBlur={(event) => handleRename(key, event.target.value)}
            aria-label={t("Key")}
          />
          <div className="min-w-0 flex-1">
            <Field
              node={node.value}
              path={childPath(path, key)}
              value={entry}
              hideLabel
              onChange={onChange}
            />
          </div>
          <Button
            className="h-6"
            size="icon"
            disabled={field.readonly}
            onClick={() => handleRemove(key)}
            aria-label={t("Remove item")}>
            <TrashIcon className="fill-red-400" />
          </Button>
        </div>
      ))}
      <Button
        size="icon"
        className="mx-0 my-2 ml-1 h-6"
        disabled={field.readonly}
        onClick={handleAdd}
        aria-label={t("Add item")}>
        <PlusIcon />
      </Button>
    </div>
  );
};

export { MapField };
