import { ArrowLeftIcon } from "@phosphor-icons/react";
import { memo, useMemo } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

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

  // Process feature properties for display
  const processedFeature = useMemo(() => {
    if (!feature) return null;
    // Separate geometry and properties for better organization
    const { geometry, ...properties } = feature;

    // Filter out internal properties that aren't user-relevant
    const filteredProperties = Object.fromEntries(
      Object.entries(properties).filter(
        ([key]) => !key.startsWith("_") && key !== "id",
      ),
    );

    return {
      id: feature.id,
      properties: filteredProperties,
      geometry,
    };
  }, [feature]);

  if (!feature || !processedFeature) {
    return null;
  }

  const formatValue = (value: any): string => {
    if (value == null || value == undefined) return "—";
    if (typeof value === "object") {
      try {
        return JSON.stringify(value, null, 2);
      } catch {
        return String(value);
      }
    }
    return value;
  };

  return (
    <div className="absolute inset-y-0 -top-10 left-0 z-10 w-full rounded-md bg-card/95 shadow-xl backdrop-blur-sm">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 border-b border-border p-2">
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
          {/* TO DO: add more toggleable/switchable features
          <Label className="text-sm font-medium">{t("Enable Texture")}</Label>
          <Switch /> */}
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

          {/* Properties */}
          {Object.keys(processedFeature.properties).length > 0 && (
            <div>
              <h4 className="mb-3 text-sm font-medium text-muted-foreground">
                {t("Attributes")}
              </h4>
              <div className="space-y-3">
                {Object.entries(processedFeature.properties).map(
                  ([key, value]) => (
                    <div key={key} className="space-y-1">
                      <div className="flex items-center justify-between">
                        <span className="text-xs font-medium text-muted-foreground">
                          {key
                            .replace(/^attributes/, "")
                            .replace(/^geometry/, "")}
                        </span>
                        {typeof value === "object" && (
                          <span className="text-xs text-muted-foreground">
                            {Array.isArray(value) ? "array" : "object"}
                          </span>
                        )}
                      </div>
                      <div className="rounded-md bg-muted/30 p-2">
                        <pre className="text-xs break-all whitespace-pre-wrap">
                          {formatValue(value)}
                        </pre>
                      </div>
                    </div>
                  ),
                )}
              </div>
            </div>
          )}

          {/* Geometry */}
          {processedFeature.geometry && (
            <div>
              <h4 className="mb-3 text-sm font-medium text-muted-foreground">
                {t("Geometry")}
              </h4>
              <div className="space-y-3">
                {Object.entries(processedFeature.geometry).map(
                  ([key, value]) => (
                    <div key={key} className="space-y-1">
                      <div className="flex items-center justify-between">
                        <span className="text-xs font-medium text-muted-foreground">
                          {key}
                        </span>
                        {typeof value === "object" && (
                          <span className="text-xs text-muted-foreground">
                            {Array.isArray(value) ? "array" : "object"}
                          </span>
                        )}
                      </div>
                      <div className="rounded-md bg-muted/30 p-2">
                        <pre className="text-xs break-all whitespace-pre-wrap">
                          {formatValue(value)}
                        </pre>
                      </div>
                    </div>
                  ),
                )}
              </div>
            </div>
          )}

          {/* No data message */}
          {Object.keys(processedFeature.properties).length === 0 &&
            !processedFeature.geometry && (
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
