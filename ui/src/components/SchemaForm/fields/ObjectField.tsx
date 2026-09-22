import { PlusIcon, TrashIcon } from "@phosphor-icons/react";
import { useCallback } from "react";

import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { applyDefaults, childPath } from "@flow/lib/schemaForm";
import type { ObjectField as ObjectFieldNode } from "@flow/lib/schemaForm";

import { Field } from "./Field";
import { SectionHeading } from "./SectionHeading";
import type { FieldProps } from "./types";
import { useField } from "./useField";

type Props = FieldProps<ObjectFieldNode> & {
  /** The form's own root, which carries the bold title and its tooltip. */
  isRoot?: boolean;
  descriptions?: Record<string, unknown>;
};

const ObjectField: React.FC<Props> = ({
  node,
  path,
  value,
  required,
  hideLabel,
  isRoot,
  descriptions,
  onChange,
}) => {
  const t = useT();
  const field = useField(path);

  const record = (value ?? {}) as Record<string, unknown>;
  const title = node.title ?? node.name;

  // An optional section stays absent until the user asks for it, and can be
  // removed again. The old form had no way to express either: patching deleted
  // the `null` branch, so every optional section rendered as mandatory and
  // half-filled, and its defaults were written back on mount.
  const isOptional = node.nullable && !required && !isRoot;
  const isPresent = value !== undefined && value !== null;

  const handleAdd = useCallback(
    () => onChange(path, applyDefaults(node, {}) ?? {}),
    [onChange, path, node],
  );
  const handleRemove = useCallback(
    () => onChange(path, undefined),
    [onChange, path],
  );

  if (isOptional && !isPresent) {
    return (
      <div className="my-1.5">
        <div className="flex items-center justify-between gap-2">
          <div className="flex flex-col">
            <p className="font-light">{title}</p>
            {node.description && (
              <span className="text-xs text-muted-foreground">
                {node.description}
              </span>
            )}
          </div>
          <Button
            className="h-6"
            size="icon"
            disabled={field.readonly}
            onClick={handleAdd}
            aria-label={t("Add {{name}}", { name: title })}>
            <PlusIcon />
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div>
      {title && !hideLabel && (
        <div className="flex items-end justify-between gap-2">
          <div className="min-w-0 flex-1">
            <SectionHeading
              id={`${field.id}-title`}
              title={title}
              required={required}
              isRoot={isRoot}
              descriptions={descriptions}
            />
          </div>
          {isOptional && (
            <Button
              className="mb-1 h-6"
              size="icon"
              disabled={field.readonly}
              onClick={handleRemove}
              aria-label={t("Remove {{name}}", { name: title })}>
              <TrashIcon className="fill-red-400" />
            </Button>
          )}
        </div>
      )}
      {node.description && !hideLabel && (
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
        {node.properties.map((property) => (
          <Field
            key={property.name}
            node={property}
            path={childPath(path, property.name)}
            value={record[property.name]}
            required={node.required.includes(property.name)}
            onChange={onChange}
          />
        ))}
      </div>
    </div>
  );
};

export { ObjectField };
