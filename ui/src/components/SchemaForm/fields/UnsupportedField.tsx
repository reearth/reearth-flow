import { WarningIcon } from "@phosphor-icons/react";
import { useCallback, useState } from "react";

import { useT } from "@flow/lib/i18n";
import type { UnsupportedField as UnsupportedFieldNode } from "@flow/lib/schemaForm";

import { FieldRow } from "./FieldRow";
import type { FieldProps } from "./types";
import { useField } from "./useField";

/**
 * The fallback for a shape the compiler does not recognise.
 *
 * Built-in actions are held to zero of these by `corpus.test.ts`; this exists
 * for schemas the UI does not control — a custom action written by hand. The
 * value stays editable as JSON rather than being dropped, so an unrecognised
 * field costs the user a nicer control, never their data.
 */
const UnsupportedField: React.FC<FieldProps<UnsupportedFieldNode>> = ({
  node,
  path,
  value,
  required,
  onChange,
}) => {
  const t = useT();
  const field = useField(path);
  const [text, setText] = useState(() =>
    value === undefined ? "" : JSON.stringify(value, null, 2),
  );
  const [parseError, setParseError] = useState<string | null>(null);

  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLTextAreaElement>) => {
      const next = event.target.value;
      setText(next);
      if (next.trim() === "") {
        setParseError(null);
        onChange(path, undefined);
        return;
      }
      try {
        onChange(path, JSON.parse(next));
        setParseError(null);
      } catch (error) {
        setParseError((error as Error).message);
      }
    },
    [onChange, path],
  );

  return (
    <FieldRow
      id={field.id}
      label={node.title ?? node.name}
      required={required}
      errors={parseError ? [parseError] : field.errors}>
      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-1 text-xs text-muted-foreground">
          <WarningIcon className="size-3.5" />
          {t("Edited as JSON: {{reason}}", { reason: node.reason })}
        </div>
        <textarea
          id={field.id}
          className={`min-h-20 w-full rounded-md border bg-background p-2 font-mono text-xs ${
            parseError || field.hasErrors ? "border-destructive" : ""
          }`}
          style={field.awarenessStyle}
          value={text}
          disabled={field.readonly}
          onChange={handleChange}
          onFocus={field.onFocus}
          onBlur={field.onBlur}
          aria-invalid={Boolean(parseError) || field.hasErrors}
        />
      </div>
    </FieldRow>
  );
};

export { UnsupportedField };
