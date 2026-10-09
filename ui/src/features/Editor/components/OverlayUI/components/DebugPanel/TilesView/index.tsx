import { MapTrifoldIcon, XIcon } from "@phosphor-icons/react";
import type { ReactNode } from "react";
import { lazy, memo, Suspense, useState } from "react";

import {
  IconButton,
  Label,
  LoadingSkeleton,
  RenderFallback,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
} from "@flow/components";
import {
  FEATURE_COLOR,
  SELECTED_COLOR,
} from "@flow/components/visualizations/featureColors";
import type {
  Heights,
  MapControls,
} from "@flow/components/visualizations/TilesViewer";
import type {
  BasemapId,
  Credit,
} from "@flow/components/visualizations/TilesViewer/sources";
import { TERRAIN_CREDITS } from "@flow/components/visualizations/TilesViewer/sources";
import { useBasemaps } from "@flow/components/visualizations/TilesViewer/useBasemaps";
import type { IntermediateDataViewError } from "@flow/lib/gql/intermediateDataView";
import { useIntermediateDataView } from "@flow/lib/gql/intermediateDataView";
import { useLang, useT } from "@flow/lib/i18n";
import type {
  IntermediateDataView,
  IntermediateDataViewRequest,
} from "@flow/types";

import Legend from "../Legend";
import { Notice, Rendering } from "../ViewStatus";

// The map engine is large, so it is fetched only when a map is first opened.
const TilesViewer = lazy(
  () => import("@flow/components/visualizations/TilesViewer"),
);

type Props = {
  request: IntermediateDataViewRequest;
  openError?: IntermediateDataViewError;
  selectedRow?: number;
  /** Shown over the map's corner, such as the selected row in brief. */
  overlay?: (map: MapState) => ReactNode;
  onPickRow: (row: number | null) => void;
  onRetry: () => void;
  onClose: () => void;
  /** Gets ways to move the map once it is ready, and loses them after. */
  onControls?: (controls: MapControls | undefined) => void;
  /** Some of the rows the table has loaded hold 2D geometry. */
  hasLoaded2D?: boolean;
};

/** What an overlay may want to know of the map under it. */
export type MapState = {
  view: IntermediateDataView;
};

/** A whole port rendered as tiles, from asking for it to showing it. */
const TilesView: React.FC<Props> = ({
  request,
  openError,
  selectedRow,
  overlay,
  onPickRow,
  onRetry,
  onClose,
  onControls,
  hasLoaded2D = false,
}) => {
  const t = useT();
  const basemaps = useBasemaps();
  const { useView, errorMessage } = useIntermediateDataView();
  const { view, error, isRendering, renderStartedAt } = useView(request);
  const [terrain, setTerrain] = useState(false);
  const [basemapId, setBasemapId] = useState<BasemapId>("paper");
  // The chosen one, or the default while the chosen one is not ready yet.
  const basemap =
    basemaps.find((candidate) => candidate.id === basemapId) ?? basemaps[0];
  const basemapLabels: Record<BasemapId, string> = {
    // Re:Earth's own map, so not translated.
    paper: "Paper",
    satellite: t("Satellite"),
  };
  // Kept with the view they came from, so another port's never show.
  // The view whose map could not be started, until the user tries again.
  const [failedUrl, setFailedUrl] = useState<string>();
  const [heightsOf, setHeightsOf] = useState<{
    url: string;
    heights: Heights;
  }>();

  const body = (() => {
    if (isRendering) {
      return (
        <Rendering
          message={t("Rendering the map…")}
          startedAt={renderStartedAt}
        />
      );
    }
    if (openError) {
      return <Notice message={errorMessage(openError)} onRetry={onRetry} />;
    }
    if (error) {
      const retryable = error.kind === "timedOut" || error.kind === "unknown";
      return (
        <Notice
          message={errorMessage(error)}
          onRetry={retryable ? onRetry : undefined}
        />
      );
    }
    if (!view) return null;

    if (view.status === "ready" && failedUrl === view.entryPointUrl) {
      return (
        <Notice
          message={t("The map could not be displayed.")}
          onRetry={() => setFailedUrl(undefined)}
        />
      );
    }
    if (
      view.status === "ready" &&
      view.entryPointUrl &&
      (view.format === "cesium3dTiles" || view.format === "vectorTiles")
    ) {
      return (
        <div className="flex h-full flex-col">
          <div className="relative min-h-0 flex-1">
            <RenderFallback
              message={t("The map could not be displayed.")}
              textSize="sm">
              <Suspense
                fallback={
                  <div className="flex h-full items-center justify-center">
                    <LoadingSkeleton />
                  </div>
                }>
                <TilesViewer
                  entryPointUrl={view.entryPointUrl}
                  format={view.format}
                  basemaps={basemaps}
                  basemap={basemap.id}
                  selectedRow={selectedRow}
                  terrain={terrain}
                  onPickRow={onPickRow}
                  onControls={onControls}
                  onError={() => setFailedUrl(view.entryPointUrl)}
                  onHeights={(heights) =>
                    view.entryPointUrl &&
                    setHeightsOf({ url: view.entryPointUrl, heights })
                  }
                />
              </Suspense>
            </RenderFallback>
            <Legend
              className="absolute top-2 left-2"
              entries={[
                { color: FEATURE_COLOR, label: t("Other features") },
                { color: SELECTED_COLOR, label: t("Selected feature") },
              ]}
            />
            {overlay?.({ view }) && (
              // Only the overlay itself takes clicks; the map around it
              // stays usable.
              <div className="pointer-events-none absolute inset-x-2 bottom-2 flex">
                {overlay({ view })}
              </div>
            )}
          </div>
          <div className="flex flex-wrap items-baseline gap-x-4 gap-y-0.5 px-2 py-1 text-xs text-muted-foreground">
            {/* 3D Tiles cannot lie on the terrain, and draw a mixed port's 2D
                shapes as flat models at height 0, beneath raised ground. */}
            {terrain && view.format === "cesium3dTiles" && hasLoaded2D && (
              <span className="text-warning">
                {t(
                  "2D shapes are drawn at height 0, so terrain can hide them.",
                )}
              </span>
            )}
            <Counts
              view={view}
              heights={
                heightsOf?.url === view.entryPointUrl
                  ? heightsOf.heights
                  : undefined
              }
            />
            <Credits
              credits={[
                ...basemap.credits,
                ...(terrain ? TERRAIN_CREDITS : []),
              ]}
            />
          </div>
        </div>
      );
    }
    if (view.status === "empty") {
      return (
        <Notice
          message={t("Nothing in this port can be drawn on a map.")}
          detail={view.error}
        />
      );
    }
    return (
      <Notice
        message={t("The map could not be rendered.")}
        detail={view.error}
        onRetry={view.status === "failed" ? onRetry : undefined}
      />
    );
  })();

  return (
    <div className="flex h-full flex-col rounded-md bg-card/60">
      <div className="flex items-center justify-between gap-2 border-b border-border p-2">
        <div className="flex items-center gap-2">
          <MapTrifoldIcon size={16} />
          <h3 className="text-sm">{t("Map")}</h3>
        </div>
        <div className="flex items-center gap-3">
          {basemaps.length > 1 && view?.status === "ready" && (
            <Select
              value={basemap.id}
              onValueChange={(id) => {
                const chosen = basemaps.find(
                  (candidate) => candidate.id === id,
                );
                if (chosen) setBasemapId(chosen.id);
              }}
              items={basemaps.map(({ id }) => ({
                value: id,
                label: basemapLabels[id],
              }))}>
              <SelectTrigger
                className="h-[26px] w-auto text-xs"
                aria-label={t("Basemap")}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {basemaps.map(({ id }) => (
                  <SelectItem key={id} value={id}>
                    {basemapLabels[id]}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
          {view?.status === "ready" && (
            <div className="flex items-center gap-2">
              <Label
                htmlFor="debug-map-terrain"
                className="text-xs font-light text-muted-foreground">
                {t("Terrain")}
              </Label>
              <Switch
                id="debug-map-terrain"
                checked={terrain}
                onCheckedChange={setTerrain}
              />
            </div>
          )}
          <IconButton
            className="h-7 w-7"
            icon={<XIcon size={16} />}
            onClick={onClose}
            tooltipText={t("Close map")}
          />
        </div>
      </div>
      <div className="min-h-0 flex-1">{body}</div>
    </div>
  );
};

/**
 * How much of the port the map shows. Fewer drawn than selected is normal;
 * features left out to keep tiles small are a separate, stronger warning,
 * since they stay missing at every zoom.
 */
const Counts: React.FC<{ view: IntermediateDataView; heights?: Heights }> = ({
  view,
  heights,
}) => {
  const t = useT();
  const lang = useLang();
  const { selectedFeatures, renderedFeatures, sizeLimitedFeatures } = view;
  if (selectedFeatures === undefined || renderedFeatures === undefined) {
    return null;
  }
  return (
    <>
      <span>
        {t("{{rendered}} of {{selected}} features drawn", {
          rendered: renderedFeatures.toLocaleString(lang),
          selected: selectedFeatures.toLocaleString(lang),
        })}
      </span>
      {heights && (
        <span>
          {t("Heights {{min}} to {{max}} m", {
            min: heights.min.toLocaleString(lang, { maximumFractionDigits: 1 }),
            max: heights.max.toLocaleString(lang, { maximumFractionDigits: 1 }),
          })}
        </span>
      )}
      {!!sizeLimitedFeatures && (
        <span className="text-warning">
          {t(
            "{{features}} features are left out where tiles reached their size limit.",
            { features: sizeLimitedFeatures.toLocaleString(lang) },
          )}
        </span>
      )}
    </>
  );
};

/** Whom the map's ground is owed to, shown where the map is. */
const Credits: React.FC<{ credits: Credit[] }> = ({ credits }) =>
  credits.length > 0 && (
    <span className="ml-auto">
      {credits.map((credit, index) => (
        <span key={credit.url + credit.label}>
          {index > 0 && " · "}
          {credit.prefix}
          <a
            href={credit.url}
            target="_blank"
            rel="noopener noreferrer"
            className="hover:underline">
            {credit.label}
          </a>
          {credit.suffix}
        </span>
      ))}
    </span>
  );

export default memo(TilesView);
