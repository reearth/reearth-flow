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
  enableClustering = false,
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
      const cycleDuration = 350;

      const animationRunId = ++animTokenRef.current;

      map.setFeatureState(key, { pulse: 1 });

      const tick = (t: number) => {
        if (animTokenRef.current !== animationRunId) return;

        const elapsed = t - start;
        if (elapsed < cycleDuration) {
          const phase = (elapsed / cycleDuration) * 2 * Math.PI;
          const pulse = 0.3 + 0.7 * (Math.sin(phase) * 0.5 + 0.5);

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
      filter: [
        "any",
        ["==", ["geometry-type"], "LineString"],
        ["==", ["geometry-type"], "MultiLineString"],
      ],
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
      filter: [
        "any",
        ["==", ["geometry-type"], "Polygon"],
        ["==", ["geometry-type"], "MultiPolygon"],
      ],
    }),
    [selectedFeatureId],
  );
  // TODO: Readd clustering support at a later date or remove entirely if deemed unnecessary
  // const clusterLayer: LayerProps = useMemo(
  //   () => ({
  //     id: "clusters",
  //     type: "circle",
  //     filter: ["has", "point_count"],
  //     paint: {
  //       "circle-color": [
  //         "step",
  //         ["get", "point_count"],
  //         "#51bbd6",
  //         100,
  //         "#f1f075",
  //         750,
  //         "#f28cb1",
  //       ],
  //       "circle-radius": ["step", ["get", "point_count"], 20, 100, 30, 750, 40],
  //     },
  //   }),
  //   [],
  // );

  // const clusterCountLayer: LayerProps = useMemo(
  //   () => ({
  //     id: "cluster-count",
  //     type: "symbol",
  //     filter: ["has", "point_count"],
  //     layout: {
  //       "text-field": "{point_count_abbreviated}",
  //       "text-size": 12,
  //     },
  //   }),
  //   [],
  // );

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

  const hasPointFeatures = useMemo(
    () => fileContent?.features?.some((f) => f.geometry.type === "Point"),
    [fileContent?.features],
  );

  const hasLineStringFeatures = useMemo(
    () =>
      fileContent?.features?.some(
        (f) =>
          f.geometry.type === "LineString" ||
          f.geometry.type === "MultiLineString",
      ),
    [fileContent?.features],
  );

  const hasPolygonFeatures = useMemo(
    () =>
      fileContent?.features?.some(
        (f) =>
          f.geometry.type === "Polygon" || f.geometry.type === "MultiPolygon",
      ),
    [fileContent?.features],
  );
  // const hasClustering =
  //   enableClustering &&
  //   hasPointFeatures &&
  //   !hasPolygonFeatures &&
  //   !hasLineStringFeatures;
  return (
    <Source
      id={SOURCE_ID}
      type={fileType}
      data={fileContent}
      cluster={enableClustering}
      promoteId="_originalId">
      {hasPointFeatures && <Layer {...pointLayer} />}
      {hasLineStringFeatures && <Layer {...lineStringLayer} />}
      {hasPolygonFeatures && <Layer {...polygonLayer} />}
      {/* <Layer {...clusterLayer} />
      <Layer {...clusterCountLayer} /> */}
    </Source>
  );
};

export { GeoJsonDataSource };
