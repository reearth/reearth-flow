import {
  ArrowDownIcon,
  ArrowUpIcon,
  CopyIcon,
  PlusIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { useCallback, useMemo } from "react";

import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { applyDefaults, childPath } from "@flow/lib/schemaForm";
import type { ArrayField as ArrayFieldNode } from "@flow/lib/schemaForm";

import { Field } from "./Field";
import { SectionHeading } from "./SectionHeading";
import type { FieldProps } from "./types";
import { useField } from "./useField";

const ArrayField: React.FC<FieldProps<ArrayFieldNode>> = ({
  node,
  path,
  value,
  required,
  onChange,
}) => {
  const t = useT();
  const field = useField(path);

  const items = useMemo(() => (Array.isArray(value) ? value : []), [value]);
  const title = node.title ?? node.name;

  const replace = useCallback(
    (next: unknown[]) => onChange(path, next),
    [onChange, path],
  );

  const handleAdd = useCallback(() => {
    // An object item starts as `{}` so its fields are there to fill in.
    // Seeding from `undefined` returns `undefined` for an object with no
    // defaults of its own, which lands in the array as a hole and reads back
    // as "must be object".
    const blank = node.item.kind === "object" ? {} : undefined;
    replace([...items, applyDefaults(node.item, blank)]);
  }, [replace, items, node.item]);

  const handleRemove = useCallback(
    (index: number) => replace(items.filter((_, i) => i !== index)),
    [replace, items],
  );

  const handleCopy = useCallback(
    (index: number) => {
      const next = [...items];
      next.splice(index + 1, 0, structuredClone(items[index]));
      replace(next);
    },
    [replace, items],
  );

  const handleMove = useCallback(
    (index: number, by: number) => {
      const target = index + by;
      if (target < 0 || target >= items.length) return;
      const next = [...items];
      [next[index], next[target]] = [next[target], next[index]];
      replace(next);
    },
    [replace, items],
  );

  const canAdd = node.maxItems === undefined || items.length < node.maxItems;
  const canRemove = node.minItems === undefined || items.length > node.minItems;

  return (
    <div>
      <SectionHeading
        id={`${field.id}-title`}
        title={title}
        required={required}
      />
      {node.description && (
        <div className="mt-1 shrink-0 text-xs text-muted-foreground">
          {node.description}
        </div>
      )}
      {field.errors && field.errors.length > 0 && (
        <div className="mt-1 text-xs text-destructive" role="alert">
          {field.errors.join(". ")}
        </div>
      )}
      <div>
        {items.map((item, index) => {
          const toolbar = (
            <div className="flex shrink-0 items-center gap-1">
              <Button
                className="h-6"
                size="icon"
                disabled={field.readonly || index === 0}
                onClick={() => handleMove(index, -1)}
                aria-label={t("Move item up")}>
                <ArrowUpIcon />
              </Button>
              <Button
                className="h-6"
                size="icon"
                disabled={field.readonly || index === items.length - 1}
                onClick={() => handleMove(index, 1)}
                aria-label={t("Move item down")}>
                <ArrowDownIcon />
              </Button>
              <Button
                className="h-6"
                size="icon"
                disabled={field.readonly || !canAdd}
                onClick={() => handleCopy(index)}
                aria-label={t("Copy item")}>
                <CopyIcon />
              </Button>
              <Button
                className="h-6"
                size="icon"
                disabled={field.readonly || !canRemove}
                onClick={() => handleRemove(index)}
                aria-label={t("Remove item")}>
                <TrashIcon className="fill-red-400" />
              </Button>
            </div>
          );

          const itemField = (
            <Field
              node={node.item}
              path={childPath(path, index)}
              value={item}
              hideLabel
              onChange={onChange}
            />
          );

          return (
            // Index is the identity here: items have no stable key of their
            // own, and reordering rewrites the whole array anyway.
            <div
              key={index}
              className="relative flex flex-col items-center rounded-md pt-2 pl-2">
              <div className="w-full">
                {node.item.kind === "object" ? (
                  <>
                    {/* An object item is named after the list it belongs to,
                        so the heading reads "Filter Conditions-1" rather than
                        repeating the raw property name on every row. */}
                    <div className="flex items-end justify-between gap-2">
                      <div className="min-w-0 flex-1">
                        <SectionHeading
                          id={`${field.id}-${index}-title`}
                          title={`${title}-${index + 1}`}
                          required={required}
                        />
                      </div>
                      {toolbar}
                    </div>
                    {itemField}
                  </>
                ) : (
                  <div className="flex items-center justify-between gap-2">
                    <div className="min-w-0 flex-1">{itemField}</div>
                    {toolbar}
                  </div>
                )}
              </div>
              <div className="w-full border-b border-primary" />
            </div>
          );
        })}
        {canAdd && (
          <Button
            size="icon"
            className="mx-0 my-2 ml-1 h-6"
            disabled={field.readonly}
            onClick={handleAdd}
            aria-label={t("Add item")}>
            <PlusIcon />
          </Button>
        )}
      </div>
    </div>
  );
};

export { ArrayField };
