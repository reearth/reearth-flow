import {
  ArrowLeftIcon,
  ArrowSquareOutIcon,
  CaretDownIcon,
} from "@phosphor-icons/react";
import { memo, useCallback, useMemo } from "react";

import {
  Button,
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  IconButton,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  feature: any;
  onClose: () => void;
  handleShowFeatureDetails?: (feature: any) => void;
  detectedGeometryType?: string | null;
};

/** Threshold for considering a value "large" — avoids JSON.stringify on huge objects */
const LARGE_VALUE_THRESHOLD = 100;

/** How many array items to show in the inline preview */
const ARRAY_PREVIEW_ITEMS = 1;

/** Maximum nesting depth for stringifyItem before truncating */
const MAX_STRINGIFY_DEPTH = 3;

/** Maximum array items to render at any depth in stringifyItem */
const MAX_ARRAY_ITEMS = 3;

/** Resolve a value that might be a JSON string into its parsed form */
function resolveValue(value: unknown): unknown {
  if (typeof value === "string") {
    try {
      const parsed = JSON.parse(value);
      if (typeof parsed === "object" && parsed !== null) return parsed;
    } catch {
      // Not valid JSON
    }
  }
  return value;
}

/** Estimate the number of leaf nodes in a value without serializing it */
function estimateSize(value: unknown): number {
  const resolved = resolveValue(value);
  if (resolved == null || typeof resolved !== "object") return 1;
  if (Array.isArray(resolved)) {
    // For large arrays, just use length — don't recurse into every element
    if (resolved.length > LARGE_VALUE_THRESHOLD) return resolved.length;
    let sum = 0;
    for (const item of resolved) {
      sum += estimateSize(item);
      if (sum > LARGE_VALUE_THRESHOLD) return sum;
    }
    return sum;
  }
  const entries = Object.entries(resolved);
  if (entries.length > LARGE_VALUE_THRESHOLD) return entries.length;
  let sum = 0;
  for (const [, v] of entries) {
    sum += estimateSize(v);
    if (sum > LARGE_VALUE_THRESHOLD) return sum;
  }
  return sum;
}

/** Stringify a value with depth limiting (used for representative items in previews).
 *  Beyond maxDepth, nested structures are shown as `Array(N)` / `Object(N keys)`. */
function stringifyItem(item: unknown, indent: string, depth = 0): string {
  if (item == null) return "null";
  if (typeof item !== "object") {
    return typeof item === "string" ? `"${item}"` : String(item);
  }
  if (Array.isArray(item)) {
    if (item.length === 0) return "[]";
    if (depth >= MAX_STRINGIFY_DEPTH) return `Array(${item.length})`;
    const shown = item.slice(0, MAX_ARRAY_ITEMS);
    const inner = shown
      .map((el) => `${indent}  ${stringifyItem(el, indent + "  ", depth + 1)}`)
      .join(",\n");
    const remaining = item.length - MAX_ARRAY_ITEMS;
    const suffix = remaining > 0 ? `,\n${indent}  ... (${remaining} more)` : "";
    return `[\n${inner}${suffix}\n${indent}]`;
  }
  const entries = Object.entries(item);
  if (entries.length === 0) return "{}";
  if (depth >= MAX_STRINGIFY_DEPTH) return `Object(${entries.length} keys)`;
  const inner = entries
    .map(
      ([k, v]) =>
        `${indent}  ${k}: ${stringifyItem(v, indent + "  ", depth + 1)}`,
    )
    .join(",\n");
  return `{\n${inner}\n${indent}}`;
}

/** Build a lightweight summary string for a large value without JSON.stringify */
function summarizeValue(value: unknown): string {
  const resolved = resolveValue(value);
  if (Array.isArray(resolved)) {
    const len = resolved.length;
    if (len === 0) return "[] (empty array)";
    // Show first item fully expanded so the user sees the complete schema
    const preview = resolved
      .slice(0, ARRAY_PREVIEW_ITEMS)
      .map((item) => stringifyItem(item, "  "))
      .join(",\n  ");
    const remaining = len - ARRAY_PREVIEW_ITEMS;
    const suffix = remaining > 0 ? `,\n  ... (${remaining} more items)` : "";
    return `Array(${len}) [\n  ${preview}${suffix}\n]`;
  }
  if (typeof resolved === "object" && resolved !== null) {
    const entries = Object.entries(resolved);
    if (entries.length === 0) return "{} (empty object)";
    const preview = entries
      .slice(0, 8)
      .map(([k, v]) => `  ${k}: ${stringifyItem(v, "  ")}`)
      .join(",\n");
    const remaining = entries.length - 8;
    const suffix = remaining > 0 ? `,\n  ... (${remaining} more keys)` : "";
    return `Object(${entries.length} keys) {\n${preview}${suffix}\n}`;
  }
  return String(resolved);
}

const FeatureDetailsOverlay: React.FC<Props> = ({
  feature,
  onClose,
  detectedGeometryType,
}) => {
  const t = useT();
  // Process feature properties for display
  const processedFeature = useMemo(() => {
    if (!feature) return null;

    const { ...properties } = feature;

    // Filter out internal properties that aren't user-relevant
    const filteredProperties = Object.fromEntries(
      Object.entries(properties).filter(
        ([key]) =>
          !key.startsWith("_") && !key.startsWith("geometry") && key !== "id",
      ),
    );

    // Filter out geometry properties
    const filteredGeometry = Object.fromEntries(
      Object.entries(properties).filter(
        ([key]) =>
          !key.startsWith("_") && key.startsWith("geometry") && key !== "id",
      ),
    );

    return {
      id: feature.id,
      attributes: filteredProperties,
      geometry: filteredGeometry,
    };
  }, [feature]);

  const openRawInNewWindow = useCallback((label: string, value: unknown) => {
    const resolved = resolveValue(value);
    const json = JSON.stringify(resolved, null, 2);
    const w = window.open("", "_blank");
    if (!w) return;
    w.document.title = label;
    const pre = w.document.createElement("pre");
    pre.style.fontFamily = "monospace";
    pre.style.fontSize = "12px";
    pre.style.padding = "16px";
    pre.style.margin = "0";
    pre.style.whiteSpace = "pre-wrap";
    pre.style.wordBreak = "break-all";
    pre.textContent = json;
    w.document.body.style.margin = "0";
    w.document.body.style.backgroundColor = "#1e1e2e";
    w.document.body.style.color = "#cdd6f4";
    w.document.body.appendChild(pre);
  }, []);

  if (!feature || !processedFeature) {
    return null;
  }

  const formatValue = (value: unknown): string => {
    if (value == null || value === undefined) return "—";

    if (typeof value === "object") {
      try {
        return JSON.stringify(value, null, 2);
      } catch {
        return String(value);
      }
    }

    if (typeof value === "string") {
      try {
        const parsed = JSON.parse(value);
        if (typeof parsed === "object" && parsed !== null) {
          return JSON.stringify(parsed, null, 2);
        }
      } catch {
        // Not valid JSON, return as-is
      }
    }

    return String(value);
  };

  const getValueType = (value: unknown): "array" | "object" | null => {
    if (typeof value === "object" && value !== null) {
      return Array.isArray(value) ? "array" : "object";
    }

    if (typeof value === "string") {
      try {
        const parsed = JSON.parse(value);
        if (typeof parsed === "object" && parsed !== null) {
          return Array.isArray(parsed) ? "array" : "object";
        }
      } catch {
        // Not valid JSON, ignore
      }
    }

    return null;
  };

  const isLargeValue = (value: unknown): boolean =>
    estimateSize(value) > LARGE_VALUE_THRESHOLD;

  const renderEntry = (
    label: string,
    value: unknown,
    valueType: "array" | "object" | null,
  ) => {
    const large = isLargeValue(value);

    return (
      <div className="space-y-1">
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium text-muted-foreground">
            {label}
          </span>
          {large && (
            <Button
              variant="ghost"
              type="button"
              className="flex h-5 items-center gap-1 px-1 text-xs text-muted-foreground hover:text-foreground"
              onClick={() => openRawInNewWindow(label, value)}>
              <ArrowSquareOutIcon size={12} />
              {t("View raw")}
            </Button>
          )}
        </div>
        {large ? (
          <div className="max-h-60 overflow-y-auto rounded-md bg-muted/30 p-2">
            <pre className="text-xs break-all whitespace-pre-wrap">
              {summarizeValue(value)}
            </pre>
          </div>
        ) : valueType === "object" || valueType === "array" ? (
          <Collapsible defaultOpen={true}>
            <CollapsibleTrigger asChild className="w-full">
              <Button
                variant="ghost"
                type="button"
                className="group flex items-center justify-between border-0 bg-transparent p-0 hover:cursor-pointer hover:bg-transparent"
                aria-expanded="true">
                <span className="group flex items-center text-xs font-medium text-muted-foreground">
                  <CaretDownIcon
                    size={12}
                    className="mr-1 transition-transform group-data-[state=open]:rotate-180"
                  />
                  {valueType}
                </span>
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div className="mt-1 rounded-md bg-muted/30 p-2">
                <pre className="text-xs break-all whitespace-pre-wrap">
                  {formatValue(value)}
                </pre>
              </div>
            </CollapsibleContent>
          </Collapsible>
        ) : (
          <div className="rounded-md bg-muted/30 p-2">
            <pre className="text-xs break-all whitespace-pre-wrap">
              {formatValue(value)}
            </pre>
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="absolute inset-y-0 -top-10 left-0 z-10 w-full rounded-md bg-card/95 shadow-xl backdrop-blur-sm">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 border-b border-border p-2 pl-0">
        <div className="flex gap-2">
          <IconButton
            className="h-7 w-7"
            icon={<ArrowLeftIcon size={16} />}
            onClick={onClose}
            tooltipText={t("Back to table")}
          />
          <div className="flex items-center gap-2">
            {detectedGeometryType && (
              <span className="text-xs text-muted-foreground">
                {detectedGeometryType}
              </span>
            )}
            <h3 className="text-sm">
              {t("Feature ID: ")} {processedFeature.id}
            </h3>
          </div>
        </div>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            type="button"
            className="flex h-7 items-center gap-1 px-2 text-xs text-muted-foreground hover:text-foreground"
            onClick={() =>
              openRawInNewWindow(`Feature ${processedFeature.id}`, feature)
            }>
            <ArrowSquareOutIcon size={12} />
            {t("View all raw")}
          </Button>
        </div>
      </div>

      {/* Content */}
      <div className="h-[calc(100%-4rem)] overflow-y-auto p-4">
        <div className="space-y-6">
          {/* Feature ID */}
          {processedFeature.id != null && (
            <div>
              <h4 className="mb-2 text-sm font-medium text-muted-foreground">
                {t("Feature ID")}
              </h4>
              <div className="rounded-md bg-muted/50 p-3">
                <code className="text-xs break-all">
                  {formatValue(processedFeature.id)}
                </code>
              </div>
            </div>
          )}
          {/* Geometry */}
          {Object.keys(processedFeature.geometry).length > 0 && (
            <div>
              <h4 className="mb-3 text-sm font-medium text-muted-foreground">
                {t("Geometry")}
              </h4>
              <div className="space-y-3">
                {Object.entries(processedFeature.geometry).map(
                  ([key, value]) => {
                    const valueType = getValueType(value);
                    const geometryKey = key.replace(/^geometry/, "");

                    return (
                      <div key={key}>
                        {renderEntry(geometryKey, value, valueType)}
                      </div>
                    );
                  },
                )}
              </div>
            </div>
          )}
          {/* Attributes */}
          {Object.keys(processedFeature.attributes).length > 0 && (
            <div>
              <h4 className="mb-3 text-sm font-medium text-muted-foreground">
                {t("Attributes")}
              </h4>
              <div className="space-y-3">
                {Object.entries(processedFeature.attributes).map(
                  ([key, value]) => {
                    const valueType = getValueType(value);
                    const attributeKey = key.replace(/^attributes/, "");

                    return (
                      <div key={key}>
                        {renderEntry(attributeKey, value, valueType)}
                      </div>
                    );
                  },
                )}
              </div>
            </div>
          )}
          {/* No data message */}
          {Object.keys(processedFeature.attributes).length === 0 &&
            Object.keys(processedFeature.geometry).length === 0 && (
              <div className="text-center text-muted-foreground">
                <p className="text-sm">
                  {t("No additional details available")}
                </p>
              </div>
            )}
        </div>
      </div>
    </div>
  );
};

export default memo(FeatureDetailsOverlay);
