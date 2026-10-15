import {
  ArrowsOutSimpleIcon,
  CrosshairSimpleIcon,
  CaretDownIcon,
  CaretUpIcon,
  CubeIcon,
} from "@phosphor-icons/react";
import { memo, useMemo, useState } from "react";

import { Button, IconButton } from "@flow/components";
import { useLang, useT } from "@flow/lib/i18n";
import { heightRange } from "@flow/lib/intermediateData";
import type { GeometrySummary } from "@flow/lib/intermediateData";
import type { IntermediateDataViewFormat } from "@flow/types";

type Props = {
  /** A table row, as the table hands it over. */
  feature: any;
  /** How the map is drawn, which decides what of the feature it can show. */
  format?: IntermediateDataViewFormat;
  onShowDetails: () => void;
  /** Flies the map to the feature, when its coordinates can place it. */
  onZoomTo?: () => void;
  /** Opens the row as a 3D model, when it can be. */
  onOpenIn3D?: () => void;
};

/**
 * The selected row as it sits on the map: its geometry in brief, how high it
 * reaches, and what the map leaves out of it. What it says is in the table
 * beside the map, and everything else is a click away in its details.
 */
const RowCard: React.FC<Props> = ({
  feature,
  format,
  onShowDetails,
  onZoomTo,
  onOpenIn3D,
}) => {
  const t = useT();
  // Collapsing keeps the card out of the way of the map. It stays collapsed
  // as other features are picked, since the card stays mounted while the
  // selection changes.
  const [collapsed, setCollapsed] = useState(false);
  const lang = useLang();
  const summary: GeometrySummary | undefined =
    feature?._values?.geometrySummary;
  const heights = useMemo(
    () => heightRange(feature?._values?.geometry),
    [feature],
  );

  const geometryLine = summary
    ? [
        summary.label,
        summary.has2D && summary.has3D
          ? t("2D and 3D")
          : summary.has3D
            ? t("3D")
            : t("2D"),
        summary.crs.map((code) => `EPSG:${code}`).join(", "),
      ]
        .filter(Boolean)
        .join(" · ")
    : t("No geometry");

  const height = (value: number) =>
    value.toLocaleString(lang, { maximumFractionDigits: 1 });

  // 3D Tiles draw only surfaces; points and lines stay out of them.
  const leftOut =
    format === "cesium3dTiles" && summary?.hasNonSurface
      ? summary.hasSurface
        ? t("Its points and lines are not drawn: 3D Tiles show only surfaces.")
        : t("Not drawn on this map: 3D Tiles show only surfaces.")
      : undefined;

  return (
    <div className="pointer-events-auto flex w-64 max-w-full flex-col gap-1 rounded-md border border-border bg-card/95 p-2 text-xs shadow-md backdrop-blur-sm">
      <div className="flex items-center justify-between gap-2">
        <span className="truncate font-medium" title={String(feature.id)}>
          {/* Not translated, to match the table's column. */}
          Feature ID: {feature.id}
        </span>
        <IconButton
          className="h-6 w-6 shrink-0"
          icon={
            collapsed ? <CaretUpIcon size={14} /> : <CaretDownIcon size={14} />
          }
          onClick={() => setCollapsed((was) => !was)}
          tooltipText={collapsed ? t("Expand") : t("Collapse")}
          aria-label={collapsed ? t("Expand") : t("Collapse")}
          aria-expanded={!collapsed}
        />
      </div>
      {!collapsed && (
        <>
          <span className="text-muted-foreground">{geometryLine}</span>
          {heights && (
            <span className="text-muted-foreground">
              {/* As the data records them. A map may draw them on another
              vertical reference, such as above the ellipsoid rather than
              above sea level, so they can differ from the map's own. */}
              {t("Heights {{min}} to {{max}} m, as recorded", {
                min: height(heights.min),
                max: height(heights.max),
              })}
            </span>
          )}
          {summary?.hasPartWithoutCrs && (
            <span className="text-warning">
              {t("Parts without a CRS are not drawn.")}
            </span>
          )}
          {leftOut && <span className="text-warning">{leftOut}</span>}
          <div className="mt-1 flex flex-wrap gap-1.5">
            <Button
              variant="outline"
              size="sm"
              className="h-6 gap-1 px-2 text-xs"
              onClick={onShowDetails}>
              <ArrowsOutSimpleIcon size={12} />
              {t("Details")}
            </Button>
            {onZoomTo && (
              <Button
                variant="outline"
                size="sm"
                className="h-6 gap-1 px-2 text-xs"
                onClick={onZoomTo}>
                <CrosshairSimpleIcon size={12} />
                {t("Zoom to")}
              </Button>
            )}
            {onOpenIn3D && (
              <Button
                variant="outline"
                size="sm"
                className="h-6 gap-1 px-2 text-xs"
                onClick={onOpenIn3D}>
                <CubeIcon size={12} />
                {t("Open in 3D")}
              </Button>
            )}
          </div>
        </>
      )}
    </div>
  );
};

export default memo(RowCard);
