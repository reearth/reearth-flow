import { useCallback, useMemo } from "react";

import { useT } from "@flow/lib/i18n";

/**
 * Display labels for the engine's diagnostic vocabulary.
 *
 * `severity`, `category` and `effectiveDisposition` are plain strings on the
 * wire, not enums, so there is no exhaustiveness check to lean on: every
 * lookup here falls back to the raw value rather than hiding a code this build
 * has not been taught about.
 */
export default () => {
  const t = useT();

  const severityLabels: Record<string, string> = useMemo(
    () => ({
      trace: t("Trace"),
      debug: t("Debug"),
      info: t("Info"),
      warn: t("Warning"),
      error: t("Error"),
      fatal: t("Fatal"),
    }),
    [t],
  );

  const dispositionLabels: Record<string, string> = useMemo(
    () => ({
      warn_drop: t("Warn and drop"),
      reject: t("Reject"),
      fatal: t("Fatal"),
    }),
    [t],
  );

  const categoryLabels: Record<string, string> = useMemo(
    () => ({
      io: t("I/O"),
      parse: t("Parse"),
      validation: t("Validation"),
      geometry: t("Geometry"),
      schema: t("Schema"),
      expression: t("Expression"),
      config: t("Config"),
      network: t("Network"),
      resource: t("Resource"),
      internal: t("Internal"),
    }),
    [t],
  );

  const statusLabels: Record<string, string> = useMemo(
    () => ({
      // `pending` is retained by the API but never emitted by the runtime; it
      // is labelled only so an unexpected value still reads sensibly.
      pending: t("Pending"),
      starting: t("Starting"),
      processing: t("Processing"),
      completed: t("Completed"),
      failed: t("Failed"),
    }),
    [t],
  );

  const severityLabel = useCallback(
    (severity: string) => severityLabels[severity] ?? severity,
    [severityLabels],
  );

  const dispositionLabel = useCallback(
    (disposition?: string) =>
      disposition ? (dispositionLabels[disposition] ?? disposition) : undefined,
    [dispositionLabels],
  );

  const categoryLabel = useCallback(
    (category: string) => categoryLabels[category] ?? category,
    [categoryLabels],
  );

  const statusLabel = useCallback(
    (status: string) => statusLabels[status] ?? status,
    [statusLabels],
  );

  return { severityLabel, dispositionLabel, categoryLabel, statusLabel };
};
