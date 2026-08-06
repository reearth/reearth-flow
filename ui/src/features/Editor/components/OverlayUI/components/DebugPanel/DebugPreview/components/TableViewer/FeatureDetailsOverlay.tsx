import {
  ArrowLeftIcon,
  BracketsCurlyIcon,
  CaretDownIcon,
} from "@phosphor-icons/react";
import {
  KeyboardEvent,
  memo,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import {
  Button,
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  IconButton,
  Input,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { AppearanceSummary } from "@flow/lib/intermediateData";
import {
  isLargeValue,
  summarizeValue,
  toSearchableString,
} from "@flow/utils/valueSummary";

import AppearanceSection from "./AppearanceSection";
import RawJsonViewer from "./RawJsonViewer";

type Props = {
  feature: any;
  onClose: () => void;
  handleShowFeatureDetails?: (feature: any) => void;
  detectedGeometryType?: string | null;
};

const FeatureDetailsOverlay: React.FC<Props> = ({
  feature,
  onClose,
  detectedGeometryType,
}) => {
  const t = useT();
  const [searchTerm, setSearchTerm] = useState<string>("");

  // Carried on the row under underscored keys, so they are not listed among
  // the feature's own fields; see useDataColumnizer.
  const appearance: AppearanceSummary | undefined = feature?._appearance;

  // The record as the engine wrote it. The fields listed below are derived for
  // the map and drop what it does not need — face holes, themes, UV sets, a
  // tangent frame's basis — so raw inspection reads this instead.
  const source: unknown = feature?._source;
  const sourceGeometry = (source as { geometry?: unknown } | undefined)
    ?.geometry;

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

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.focus({ preventScroll: true });
    }
  }, []);

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
        event.preventDefault();
        onClose();
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
            <pre className="text-xs break-all whitespace-pre-wrap">
              {summarizeValue(value)}
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
    <div className="absolute inset-0 z-10 rounded-md bg-card/95 shadow-xl backdrop-blur-sm">
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
              openRaw(
                `${t("Feature")} ${processedFeature.id}`,
                source ?? feature,
              )
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
          {Object.keys(filteredFeature?.geometry || {}).length > 0 && (
            <div>
              <div className="mb-3 flex items-center justify-between">
                <h4 className="text-sm font-medium text-muted-foreground">
                  {t("Geometry")}
                </h4>
                {sourceGeometry != null && (
                  <Button
                    variant="ghost"
                    type="button"
                    className="flex h-5 items-center gap-1 px-1 text-xs text-muted-foreground hover:text-foreground"
                    onClick={() =>
                      openRaw(t("Geometry (as written)"), sourceGeometry)
                    }>
                    <BracketsCurlyIcon size={12} />
                    {t("View structure")}
                  </Button>
                )}
              </div>
              <div className="space-y-3">
                {Object.entries(
                  (filteredFeature?.geometry ?? {}) as Record<string, unknown>,
                ).map(([key, value]) => {
                  const valueType = getValueType(value);
                  const geometryKey = key.replace(/^geometry/, "");

                  return (
                    <div key={key}>
                      {renderEntry(geometryKey, value, valueType)}
                    </div>
                  );
                })}
              </div>
            </div>
          )}
          {/* Appearance */}
          {appearance && !searchTerm && (
            <AppearanceSection appearance={appearance} />
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
                ).map(([key, value]) => {
                  const valueType = getValueType(value);
                  const attributeKey = key.replace(/^attributes/, "");

                  return (
                    <div key={key}>
                      {renderEntry(attributeKey, value, valueType)}
                    </div>
                  );
                })}
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

export default memo(FeatureDetailsOverlay);
