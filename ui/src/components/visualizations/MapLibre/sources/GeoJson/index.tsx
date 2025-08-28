import type { Map } from "maplibre-gl";
import { useCallback, useEffect, useMemo, useRef } from "react";
import { Source, Layer, LayerProps } from "react-map-gl/maplibre";

type Props = {
  fileType: "geojson";
  fileContent: GeoJSON.FeatureCollection<
    GeoJSON.Geometry,
    GeoJSON.GeoJsonProperties
  >;
  mapRef: React.RefObject<maplibregl.Map | null>;
  enableClustering?: boolean;
  selectedFeatureId?: string;
};

const SOURCE_ID = "geojson-data-source";

const GeoJsonDataSource: React.FC<Props> = ({
  fileType,
  fileContent,
  mapRef,
  enableClustering,
  selectedFeatureId,
}) => {
  const requestAnimationFrameRef = useRef<number | null>(null);
  const endTimeoutRef = useRef<number | null>(null);
  const prevSelectedRef = useRef<string | undefined>(undefined);
  const animTokenRef = useRef<number>(0);

  const clearTimers = useCallback(() => {
    if (requestAnimationFrameRef.current)
      cancelAnimationFrame(requestAnimationFrameRef.current);
    if (endTimeoutRef.current) clearTimeout(endTimeoutRef.current);
    requestAnimationFrameRef.current = null;
    endTimeoutRef.current = null;
  }, []);

  const runSelectAnimation = useCallback(
    (map: Map) => {
      clearTimers();
      if (
        prevSelectedRef.current &&
        prevSelectedRef.current !== selectedFeatureId
      ) {
        map.removeFeatureState({
          source: SOURCE_ID,
          id: prevSelectedRef.current,
        });
      }

      if (!selectedFeatureId) return;

      prevSelectedRef.current = selectedFeatureId;

      const key = { source: SOURCE_ID, id: selectedFeatureId };
      const start = performance.now();
      const flashDuration = 300;
      const cycleMs = 175;

      const animationRunId = ++animTokenRef.current;

      map.setFeatureState(key, { pulse: 1 });

      const tick = (t: number) => {
        if (animTokenRef.current !== animationRunId) return;

        const elapsed = t - start;
        if (elapsed < flashDuration) {
          const phase = (elapsed % cycleMs) / cycleMs;
          const pulse = 0.3 + 0.7 * (Math.sin(phase * Math.PI * 2) * 0.5 + 0.5);

          map.setFeatureState(key, { pulse });
          requestAnimationFrameRef.current = requestAnimationFrame(tick);
        } else {
          map.setFeatureState(key, { pulse: 1 });
        }
      };

      requestAnimationFrameRef.current = requestAnimationFrame(tick);
    },
    [clearTimers, selectedFeatureId],
  );

  const pointLayer: LayerProps = useMemo(
    () => ({
      id: "point-layer",
      type: "circle",
      filter: [
        "all",
        ["==", ["geometry-type"], "Point"],
        ["!", ["has", "point_count"]],
      ],
      paint: {
        "circle-radius": 4,
        "circle-color": "#3f3f45",
        "circle-stroke-width": 5,
        "circle-stroke-color": "rgba(0, 163, 64, 0.6)",
        "circle-stroke-opacity": [
          "case",
          ["==", ["get", "_originalId"], selectedFeatureId ?? ""],
          [
            "interpolate",
            ["linear"],
            ["coalesce", ["feature-state", "pulse"], 0],
            0,
            0.0,
            1,
            1.0,
          ],
          0.0,
        ],
      },
    }),
    [selectedFeatureId],
  );

  const lineStringLayer: LayerProps = useMemo(
    () => ({
      id: "line-layer",
      type: "line",
      paint: {
        "line-color": selectedFeatureId
          ? [
              "case",
              ["==", ["get", "_originalId"], selectedFeatureId],
              "#00a340",
              "#3f3f45",
            ]
          : "#3f3f45",
        "line-width": selectedFeatureId
          ? ["case", ["==", ["get", "_originalId"], selectedFeatureId], 4, 2]
          : 2,
      },
      filter: ["==", ["geometry-type"], "LineString"],
    }),
    [selectedFeatureId],
  );

  const polygonLayer: LayerProps = useMemo(
    () => ({
      id: "polygon-layer",
      type: "fill",
      paint: {
        "fill-color": selectedFeatureId
          ? [
              "case",
              ["==", ["get", "_originalId"], selectedFeatureId],
              "#00a340",
              "#3f3f45",
            ]
          : "#3f3f45",
        "fill-opacity": selectedFeatureId
          ? [
              "case",
              ["==", ["get", "_originalId"], selectedFeatureId],
              0.9,
              0.8,
            ]
          : 0.8,
      },
      filter: ["==", ["geometry-type"], "Polygon"],
    }),
    [selectedFeatureId],
  );

  const clusterLayer: LayerProps = useMemo(
    () => ({
      id: "clusters",
      type: "circle",
      filter: ["has", "point_count"],
      paint: {
        "circle-color": [
          "step",
          ["get", "point_count"],
          "#51bbd6",
          100,
          "#f1f075",
          750,
          "#f28cb1",
        ],
        "circle-radius": ["step", ["get", "point_count"], 20, 100, 30, 750, 40],
      },
    }),
    [],
  );

  const clusterCountLayer: LayerProps = useMemo(
    () => ({
      id: "cluster-count",
      type: "symbol",
      filter: ["has", "point_count"],
      layout: {
        "text-field": "{point_count_abbreviated}",
        "text-size": 12,
      },
    }),
    [],
  );

  useEffect(() => {
    const map = mapRef.current;
    if (!map) return;
    if (!selectedFeatureId) {
      clearTimers();
      if (prevSelectedRef.current) {
        map.removeFeatureState({
          source: SOURCE_ID,
          id: prevSelectedRef.current,
        });
      }
      prevSelectedRef.current = undefined;
      return;
    }

    if (!map.isStyleLoaded()) {
      const onStyle = () => {
        map.off("styledata", onStyle);
        runSelectAnimation(map);
      };
      map.on("styledata", onStyle);
      return () => map.off("styledata", onStyle);
    }
    runSelectAnimation(map);

    return () => {
      clearTimers();
    };
  }, [mapRef, selectedFeatureId, clearTimers, runSelectAnimation]);

  return (
    <Source
      id={SOURCE_ID}
      type={fileType}
      data={fileContent}
      cluster={enableClustering}
      promoteId="_originalId">
      {fileContent?.features?.some(
        (feature: GeoJSON.Feature) => feature.geometry.type === "Point",
      ) && <Layer {...pointLayer} />}
      {fileContent?.features?.some(
        (feature: GeoJSON.Feature) => feature.geometry.type === "LineString",
      ) && <Layer {...lineStringLayer} />}
      {fileContent?.features?.some(
        (feature: GeoJSON.Feature) => feature.geometry.type === "Polygon",
      ) && <Layer {...polygonLayer} />}
      {enableClustering && <Layer {...clusterLayer} />}
      {enableClustering && <Layer {...clusterCountLayer} />}
    </Source>
  );
};

export { GeoJsonDataSource };
