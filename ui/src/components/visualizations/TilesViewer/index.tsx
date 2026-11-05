import ThreeView, { Color } from "@navaramap/three";
import type { FeatureEvaluator, Layer } from "@navaramap/three";
import { DefaultPlugin } from "@navaramap/three-default-plugin";
import type { DefaultDescriptions } from "@navaramap/three-default-plugin";
import { memo, useEffect, useRef } from "react";

import type { IntermediateDataViewFormat } from "@flow/types";

import { FEATURE_COLOR, SELECTED_COLOR } from "../featureColors";

import { toExplicitTileset } from "./explicitTileset";
import type { Basemap, BasemapId } from "./sources";
import { TERRAIN_MAX_ZOOM, TERRAIN_URL } from "./sources";

const NO_BASEMAPS: Basemap[] = [];

const ROW_INDEX = "rowIndex";
const VECTOR_LAYER = "features";
// Polygons are slightly see-through, to show what is beneath them. Only
// polygons: they lie flat on the ground, so nothing overlaps out of order.
const POLYGON_OPACITY = 0.7;

/** One rendered layer of a view, as the map loads it. */
export type TilesLayer = {
  format: Extract<IntermediateDataViewFormat, "cesium3dTiles" | "vectorTiles">;
  entryPointUrl: string;
};

type Props = {
  /**
   * What to draw, one layer per format: a port holding both 3D and 2D geometry
   * has 3D Tiles for its 3D features and vector tiles for its 2D ones, both
   * carrying the row. Rebuilds the map when the list changes.
   */
  layers: TilesLayer[];
  /** What can be drawn beneath the data. Rebuilds the map when it changes. */
  basemaps?: Basemap[];
  /** Which of `basemaps` is shown; the first when unset. */
  basemap?: BasemapId;
  /** The table row to highlight. */
  selectedRow?: number;
  /**
   * Draws the ground's shape; otherwise the ground is flat at height 0.
   * Vector tiles are draped onto it; 3D Tiles stay at their own heights.
   */
  terrain?: boolean;
  onPickRow?: (row: number | null) => void;
  /**
   * The lowest and highest heights the 3D Tiles reach, when there are any.
   * Vector tiles lie on the ground, so they have none of their own.
   */
  onHeights?: (heights: Heights) => void;
  /** Gets ways to move the map once it is ready, and loses them after. */
  onControls?: (controls: MapControls | undefined) => void;
  /** The map could not be started, or its content could not be read. */
  onError?: (error: unknown) => void;
};

/** A place to fly to: an area in degrees, and the heights it spans. */
export type FlyTarget = Area & { minHeight?: number; maxHeight?: number };

export type MapControls = {
  /** Flies the camera to look at the whole of `target`. */
  flyTo: (target: FlyTarget) => void;
};

/** How long a flight to something on the map takes, in milliseconds. */
const FLIGHT_MS = 1000;

export type Heights = { min: number; max: number };

export type Area = {
  west: number;
  south: number;
  east: number;
  north: number;
};

const EARTH_RADIUS = 6_371_000;

/** How far back to stand to see all of `area`, in metres. */
const distanceToSee = ({ west, south, east, north }: Area) => {
  const midLat = ((south + north) / 2) * (Math.PI / 180);
  const width =
    (east - west) * (Math.PI / 180) * EARTH_RADIUS * Math.cos(midLat);
  const height = (north - south) * (Math.PI / 180) * EARTH_RADIUS;
  return Math.max(width, height, 50) * 1.8;
};

/** How the camera looks at what it flies to: from the south, 45° down. */
const cameraToSee = ({
  minHeight = 0,
  maxHeight = minHeight,
  ...area
}: FlyTarget) => ({
  lng: (area.west + area.east) / 2,
  lat: (area.south + area.north) / 2,
  height: (minHeight + maxHeight) / 2,
  heading: 0,
  pitch: -45,
  distance: Math.max(distanceToSee(area), (maxHeight - minHeight) * 2),
});

const rowOf = (properties: Record<string, unknown> | undefined) => {
  const value = Number(properties?.[ROW_INDEX]);
  return Number.isFinite(value) ? value : null;
};

/** The smallest target that takes in every one of `targets`. */
const unionOf = (targets: FlyTarget[]): FlyTarget | null => {
  if (targets.length === 0) return null;
  const heights = targets.flatMap(({ minHeight, maxHeight }) =>
    [minHeight, maxHeight].filter((h): h is number => h !== undefined),
  );
  return {
    west: Math.min(...targets.map((t) => t.west)),
    south: Math.min(...targets.map((t) => t.south)),
    east: Math.max(...targets.map((t) => t.east)),
    north: Math.max(...targets.map((t) => t.north)),
    ...(heights.length > 0 && {
      minHeight: Math.min(...heights),
      maxHeight: Math.max(...heights),
    }),
  };
};

/** A 3D Tiles region (radians) as an area in degrees. */
const areaOfRegion = (region: unknown): Area | null => {
  if (!Array.isArray(region)) return null;
  const toDeg = (r: number) => (r * 180) / Math.PI;
  return {
    west: toDeg(region[0]),
    south: toDeg(region[1]),
    east: toDeg(region[2]),
    north: toDeg(region[3]),
  };
};

/**
 * A rendered view of a whole port, as 3D Tiles, vector tiles or both, on a
 * globe. Features are drawn in one colour and the selected row in another, in
 * every layer; picking a feature in any layer reports its row. The map is built
 * once per view and basemap list; the selection, basemap and terrain change
 * without rebuilding it.
 */
const TilesViewer: React.FC<Props> = ({
  layers,
  basemaps = NO_BASEMAPS,
  basemap,
  selectedRow,
  terrain = false,
  onPickRow,
  onHeights,
  onControls,
  onError,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<ThreeView<DefaultDescriptions>>(undefined);
  const dataLayersRef = useRef<Layer[]>([]);
  const terrainRef = useRef(terrain);
  const basemapRef = useRef(basemap);
  const showBasemapRef = useRef<(id?: BasemapId) => void>(undefined);
  const terrainLayerRef = useRef<Layer>(undefined);
  const onHeightsRef = useRef(onHeights);
  const onControlsRef = useRef(onControls);
  const onErrorRef = useRef(onError);
  const selectedRowRef = useRef(selectedRow);
  const onPickRowRef = useRef(onPickRow);

  useEffect(() => {
    onPickRowRef.current = onPickRow;
    onHeightsRef.current = onHeights;
    onControlsRef.current = onControls;
    onErrorRef.current = onError;
  }, [onPickRow, onHeights, onControls, onError]);

  // Adds or removes the terrain to match `terrainRef`, once the view is up.
  const applyTerrain = () => {
    const view = viewRef.current;
    if (!view) return;
    if (terrainRef.current && !terrainLayerRef.current) {
      const source = view.addSource({
        type: "quantized-mesh",
        url: TERRAIN_URL,
        maxZoom: TERRAIN_MAX_ZOOM,
        requestVertexNormals: true,
      });
      terrainLayerRef.current = view.addLayer({ type: "terrain", source });
    } else if (!terrainRef.current && terrainLayerRef.current) {
      terrainLayerRef.current.delete();
      terrainLayerRef.current = undefined;
    }
    view.forceUpdate();
  };

  // Hidden rather than removed, so it stays beneath the data when shown again.
  useEffect(() => {
    basemapRef.current = basemap;
    showBasemapRef.current?.(basemap);
  }, [basemap]);

  useEffect(() => {
    terrainRef.current = terrain;
    applyTerrain();
  }, [terrain]);

  // Styles are applied by the engine on its own update, so a change of
  // selection asks it to apply them again rather than applying them here.
  useEffect(() => {
    selectedRowRef.current = selectedRow;
    for (const layer of dataLayersRef.current) layer.forceUpdate();
    viewRef.current?.forceUpdate();
  }, [selectedRow]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    // The page must not scroll while the wheel zooms the map.
    const keepWheel = (e: WheelEvent) => e.preventDefault();
    container.addEventListener("wheel", keepWheel, { passive: false });

    let view: ThreeView<DefaultDescriptions> | undefined;
    const tilesetUrls: string[] = [];
    let disposed = false;

    const restyle = (evaluator: FeatureEvaluator) => {
      const selected = selectedRowRef.current;
      // Opacity goes with the colour: a styled colour alone leaves the
      // feature opaque.
      evaluator.evaluate(({ properties, meshGeomType }) => ({
        color: new Color().setStyle(
          selected !== undefined && rowOf(properties) === selected
            ? SELECTED_COLOR
            : FEATURE_COLOR,
        ),
        ...(meshGeomType === "polygon" && { opacity: POLYGON_OPACITY }),
      }));
    };

    const track = (layer: Layer) => {
      layer.on("featureCreated", ({ evaluator }) => restyle(evaluator));
      layer.on("featureUpdated", ({ evaluator }) => restyle(evaluator));
      dataLayersRef.current.push(layer);
    };

    const start = async () => {
      // The canvas follows the container's size (below), not the window's.
      const created = new ThreeView<DefaultDescriptions>({
        container,
        disableAutoResize: true,
        defaultAttribution: false,
      });
      view = created;
      created.addPlugin(new DefaultPlugin());
      await created.init();
      if (disposed) return;
      viewRef.current = created;

      created.addLight({ ambient: { intensity: 1.5 } });
      created.addLight({ sun: {} });

      applyTerrain();

      // Every basemap is added beneath the data, and only the chosen one is
      // shown: one added later would be drawn over the data.
      const basemapLayers = basemaps.map((candidate) => {
        const source = created.addSource({
          type: "raster-tile",
          url: candidate.url,
          maxZoom: candidate.maxZoom,
        });
        const isShown = (id?: BasemapId) =>
          candidate.id === (id ?? basemaps[0]?.id);
        const layer = created.addLayer({
          type: "raster",
          source,
          raster: { show: isShown(basemapRef.current) },
        });
        return { layer, source, isShown };
      });
      showBasemapRef.current = (id) => {
        for (const { layer, source, isShown } of basemapLayers) {
          layer.update({
            type: "raster",
            source,
            raster: { show: isShown(id) },
          });
        }
        created.forceUpdate();
      };

      onControlsRef.current?.({
        flyTo: (target) => {
          created.flyTo(cameraToSee(target), { duration: FLIGHT_MS });
        },
      });

      created.on("featureClick", (info) => {
        onPickRowRef.current?.(rowOf(info?.properties));
      });

      const areas: FlyTarget[] = [];
      for (const layer of layers) {
        if (layer.format === "cesium3dTiles") {
          const tileset = await toExplicitTileset(layer.entryPointUrl);
          if (disposed) return;
          const tilesetUrl = URL.createObjectURL(
            new Blob([JSON.stringify(tileset)], { type: "application/json" }),
          );
          tilesetUrls.push(tilesetUrl);
          const source = created.addSource({
            type: "3d-tiles",
            url: tilesetUrl,
          });
          track(created.addLayer({ type: "3d-tiles", source, model: {} }));
          const region = tileset.root?.boundingVolume?.region;
          const regionArea = areaOfRegion(region);
          if (regionArea) {
            areas.push({
              ...regionArea,
              minHeight: region[4],
              maxHeight: region[5],
            });
          }
          if (Array.isArray(region)) {
            onHeightsRef.current?.({ min: region[4], max: region[5] });
          }
        } else {
          const response = await fetch(layer.entryPointUrl);
          if (!response.ok) {
            throw new Error(`${response.status} for the view's TileJSON`);
          }
          const tilejson = await response.json();
          if (disposed) return;
          const source = created.addSource({
            type: "vector-tile",
            url: tilejson.tiles[0],
            minZoom: tilejson.minzoom,
            maxZoom: tilejson.maxzoom,
          });
          const color = new Color().setStyle(FEATURE_COLOR);
          track(
            created.addLayer({
              type: "vector",
              source,
              sourceLayers: [VECTOR_LAYER],
              polygon: { color, opacity: POLYGON_OPACITY, clampToGround: true },
              polyline: { color, width: 3, clampToGround: true },
              // Sized on screen, and never hidden behind a neighbour: a debug
              // view must show every feature. Not clamped: clamped points are
              // baked into the ground's tiles, where the engine loses some of
              // them at some zooms.
              point: {
                color,
                size: 8,
                sizeInMeters: false,
                declutter: false,
                clampToGround: false,
              },
            }),
          );
          const [west, south, east, north] = tilejson.bounds ?? [];
          if ([west, south, east, north].every(Number.isFinite)) {
            areas.push({ west, south, east, north });
          }
        }
      }

      const area = unionOf(areas);
      if (area) created.flyTo(cameraToSee(area), { duration: 0 });
    };

    // Start once the container has a size; the engine cannot start without
    // one, and a pane that is still opening may briefly have none.
    const observer = new ResizeObserver(([entry]) => {
      const { width, height } = entry.contentRect;
      if (!width || !height) return;
      if (view) {
        view.resize(width, height, window.devicePixelRatio);
        return;
      }
      start().catch((err) => {
        if (!disposed) onErrorRef.current?.(err);
      });
    });
    observer.observe(container);

    return () => {
      disposed = true;
      observer.disconnect();
      container.removeEventListener("wheel", keepWheel);
      onControlsRef.current?.(undefined);
      dataLayersRef.current = [];
      terrainLayerRef.current = undefined;
      showBasemapRef.current = undefined;
      viewRef.current = undefined;
      view?.dispose();
      for (const tilesetUrl of tilesetUrls) URL.revokeObjectURL(tilesetUrl);
    };
  }, [layers, basemaps]);

  // Positioned so the container's size never depends on the canvas inside it.
  return (
    <div className="relative h-full w-full overflow-hidden">
      <div
        ref={containerRef}
        className="absolute inset-0 overscroll-contain [&>canvas]:block"
      />
    </div>
  );
};

export default memo(TilesViewer);
