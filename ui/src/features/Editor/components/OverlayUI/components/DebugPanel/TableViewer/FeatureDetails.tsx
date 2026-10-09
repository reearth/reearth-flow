import {
  ArrowLeftIcon,
  BracketsCurlyIcon,
  CaretDownIcon,
} from "@phosphor-icons/react";
import type { KeyboardEvent, ReactNode } from "react";
import { memo, useCallback, useMemo, useRef, useState } from "react";

import {
  Button,
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  IconButton,
  Input,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { GeometrySummary } from "@flow/lib/intermediateData";
import {
  formatStructured,
  isLargeValue,
  toSearchableString,
} from "@flow/utils/valueSummary";

import RawJsonViewer from "./RawJsonViewer";

type Props = {
  feature: any;
  /** Things the user can do with the feature, shown in the header. */
  actions?: ReactNode;
  /** Leaves the details, by a back button in the header named by `label`. */
  back?: { label: string; onClick: () => void };
};

const FeatureDetails: React.FC<Props> = ({ feature, actions, back }) => {
  const t = useT();
  const [searchTerm, setSearchTerm] = useState<string>("");

  // Read the values behind the row rather than the row's own cell strings: a
  // cell string is serialized to fit a cell, and a large one is a bounded
  // preview that no longer parses back into the value it came from. See
  // `_values` in `useDataColumnizer`.
  const processedFeature = useMemo(() => {
    if (!feature) return null;

    const values = feature._values as
      | {
          geometry?: Record<string, unknown>;
          attributes?: Record<string, unknown>;
        }
      | undefined;

    return {
      id: feature.id,
      attributes: values?.attributes ?? {},
      geometry: values?.geometry ?? {},
      geometrySummary: (values as { geometrySummary?: GeometrySummary })
        ?.geometrySummary,
    };
  }, [feature]);

  const filteredFeature = useMemo(() => {
    if (!processedFeature) return null;
    if (!searchTerm) return processedFeature;

    const lowerSearch = searchTerm.toLowerCase();

    const filteredAttributes = Object.fromEntries(
      Object.entries(processedFeature?.attributes || {}).filter(
        ([key, value]) => {
          const keyMatch = key.toLowerCase().includes(lowerSearch);
          const valueMatch = toSearchableString(value)
            .toLowerCase()
            .includes(lowerSearch);
          return keyMatch || valueMatch;
        },
      ),
    );

    const filteredGeometry = Object.fromEntries(
      Object.entries(processedFeature?.geometry || {}).filter(
        ([key, value]) => {
          const keyMatch = key.toLowerCase().includes(lowerSearch);
          const valueMatch = toSearchableString(value)
            .toLowerCase()
            .includes(lowerSearch);
          return keyMatch || valueMatch;
        },
      ),
    );

    return {
      ...processedFeature,
      attributes: filteredAttributes,
      geometry: filteredGeometry,
    };
  }, [processedFeature, searchTerm]);

  const scrollRef = useRef<HTMLDivElement | null>(null);

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const { current } = scrollRef;
    if (!current) return;

    const scrollAmount = 50;

    switch (event.key) {
      case "ArrowUp":
        event.preventDefault();
        current.scrollBy({ top: -scrollAmount, behavior: "smooth" });
        break;
      case "ArrowDown":
        event.preventDefault();
        current.scrollBy({ top: scrollAmount, behavior: "smooth" });
        break;
      case "ArrowLeft":
        if (!back) break;
        event.preventDefault();
        back.onClick();
        break;
      default:
        break;
    }
  };
  const [rawView, setRawView] = useState<{
    label: string;
    value: unknown;
  } | null>(null);

  const openRaw = useCallback((label: string, value: unknown) => {
    setRawView({ label, value });
  }, []);

  if (!feature || !processedFeature) {
    return null;
  }

  const formatValue = (value: unknown): string => {
    if (value == null || value === undefined) return "—";

    if (typeof value === "object") return formatStructured(value);

    if (typeof value === "string") {
      try {
        const parsed = JSON.parse(value);
        if (typeof parsed === "object" && parsed !== null) {
          return formatStructured(parsed);
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
              onClick={() => openRaw(label, value)}>
              <BracketsCurlyIcon size={12} />
              {t("View raw")}
            </Button>
          )}
        </div>
        {large ? (
          <div className="max-h-60 overflow-y-auto rounded-md bg-muted/30 p-2">
            {/* `wrap-break-word`, never `break-all`: the latter wraps mid-token,
                which splits a coordinate across lines in the middle of a
                number and makes the value unreadable. */}
            <pre className="text-xs wrap-break-word whitespace-pre-wrap">
              {formatValue(value)}
            </pre>
          </div>
        ) : valueType === "object" || valueType === "array" ? (
          <Collapsible defaultOpen={true}>
            <CollapsibleTrigger
              className="w-full"
              render={
                <Button
                  variant="ghost"
                  type="button"
                  className="group flex items-center justify-between border-0 bg-transparent p-0 hover:cursor-pointer hover:bg-transparent"
                  aria-expanded="true">
                  <span className="group flex items-center text-xs font-medium text-muted-foreground">
                    <CaretDownIcon
                      size={12}
                      className="mr-1 transition-transform group-data-[panel-open]:rotate-180"
                    />
                    {valueType}
                  </span>
                </Button>
              }
            />
            <CollapsibleContent>
              <div className="mt-1 rounded-md bg-muted/30 p-2">
                <pre className="text-xs wrap-break-word whitespace-pre-wrap">
                  {formatValue(value)}
                </pre>
              </div>
            </CollapsibleContent>
          </Collapsible>
        ) : (
          <div className="rounded-md bg-muted/30 p-2">
            <pre className="text-xs wrap-break-word whitespace-pre-wrap">
              {formatValue(value)}
            </pre>
          </div>
        )}
      </div>
    );
  };

  return (
    // It stands beside the table, whose arrow keys move the selection it
    // shows, so it never takes the keyboard.
    <div className="relative h-full rounded-md bg-card/60">
      {/* Header */}
      <div className="py-1">
        <Input
          placeholder={t("Search") + "..."}
          value={searchTerm}
          onChange={(e) => {
            const value = String(e.target.value);
            setSearchTerm(value);
          }}
          className="max-w-sm"
        />
      </div>

      <div className="flex items-center justify-between gap-2 border-b border-border p-2 pl-0">
        <div className="flex gap-2">
          {back && (
            <IconButton
              className="h-7 w-7"
              icon={<ArrowLeftIcon size={16} />}
              onClick={back.onClick}
              tooltipText={back.label}
            />
          )}
          <div className="flex items-center gap-2">
            <h3 className="text-sm">Feature ID: {processedFeature.id}</h3>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {actions}
          <Button
            variant="ghost"
            type="button"
            className="flex h-7 items-center gap-1 px-2 text-xs text-muted-foreground hover:text-foreground"
            onClick={() =>
              openRaw(`${t("Feature")} ${processedFeature.id}`, {
                id: processedFeature.id,
                geometry: processedFeature.geometry,
                attributes: processedFeature.attributes,
              })
            }>
            <BracketsCurlyIcon size={12} />
            {t("View all raw")}
          </Button>
        </div>
      </div>

      {/* Content */}
      <div
        className="h-[calc(100%-4rem)] overflow-y-auto p-4 focus-visible:outline-hidden"
        ref={scrollRef}
        onKeyDown={handleKeyDown}>
        <div className="space-y-6">
          {/* Feature ID */}
          {processedFeature.id != null && (
            <div>
              <h4 className="mb-2 text-sm font-medium text-muted-foreground">
                Feature ID
              </h4>
              <div className="rounded-md bg-muted/50 p-3">
                <code className="text-xs break-all">
                  {formatValue(processedFeature.id)}
                </code>
              </div>
            </div>
          )}
          {processedFeature.geometrySummary && (
            <GeometrySummaryCard summary={processedFeature.geometrySummary} />
          )}
          {/* Geometry */}
          {Object.keys(filteredFeature?.geometry || {}).length > 0 && (
            <div>
              <div className="mb-3 flex items-center justify-between">
                <h4 className="text-sm font-medium text-muted-foreground">
                  {t("Geometry")}
                </h4>
              </div>
              <div className="space-y-3">
                {Object.entries(
                  (filteredFeature?.geometry ?? {}) as Record<string, unknown>,
                ).map(([key, value]) => (
                  <div key={key}>
                    {renderEntry(key, value, getValueType(value))}
                  </div>
                ))}
              </div>
            </div>
          )}
          {/* Attributes */}
          {Object.keys(filteredFeature?.attributes || {}).length > 0 && (
            <div>
              <h4 className="mb-3 text-sm font-medium text-muted-foreground">
                {t("Attributes")}
              </h4>
              <div className="space-y-3">
                {Object.entries(
                  (filteredFeature?.attributes ?? {}) as Record<
                    string,
                    unknown
                  >,
                ).map(([key, value]) => (
                  <div key={key}>
                    {renderEntry(key, value, getValueType(value))}
                  </div>
                ))}
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

      {rawView && (
        <RawJsonViewer
          label={rawView.label}
          value={rawView.value}
          open={!!rawView}
          onClose={() => setRawView(null)}
        />
      )}
    </div>
  );
};

/**
 * What the row's geometry is, and what a rendered view can make of it, read
 * from the engine's own form rather than the converted one below it.
 */
const GeometrySummaryCard: React.FC<{ summary: GeometrySummary }> = ({
  summary,
}) => {
  const t = useT();
  const dimension =
    summary.has2D && summary.has3D
      ? t("2D and 3D")
      : summary.has3D
        ? t("3D")
        : t("2D");

  return (
    <div>
      <h4 className="mb-2 text-sm font-medium text-muted-foreground">
        {t("Geometry summary")}
      </h4>
      <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 rounded-md bg-muted/50 p-3 text-xs">
        {summary.label && (
          <>
            <dt className="text-muted-foreground">{t("Type")}</dt>
            <dd>{summary.label}</dd>
          </>
        )}
        <dt className="text-muted-foreground">{t("Dimension")}</dt>
        <dd>{dimension}</dd>
        <dt className="text-muted-foreground">{t("CRS")}</dt>
        <dd>
          {summary.crs.length
            ? summary.crs.map((code) => `EPSG:${code}`).join(", ")
            : t("None")}
        </dd>
      </dl>
      {summary.hasPartWithoutCrs && (
        <p className="mt-2 text-xs text-warning">
          {summary.crs.length
            ? t(
                "Part of this geometry has no CRS, so rendered views leave that part out.",
              )
            : t(
                "This geometry has no CRS, so rendered views cannot place it and leave it out.",
              )}
        </p>
      )}
    </div>
  );
};

export default memo(FeatureDetails);
