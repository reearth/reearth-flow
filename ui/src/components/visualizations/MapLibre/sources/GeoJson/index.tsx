import { useCallback, useMemo } from "react";
import { Source, Layer, LayerProps } from "react-map-gl/maplibre";

type Props = {
  fileType: "geojson";
  fileContent: GeoJSON.FeatureCollection<
    GeoJSON.Geometry,
    GeoJSON.GeoJsonProperties
  >;
  enableClustering?: boolean;
  selectedFeatureId?: string;
};

const GeoJsonDataSource: React.FC<Props> = ({
  fileType,
  fileContent,
  enableClustering,
  selectedFeatureId,
}) => {
  const processOverlappingPoints = useCallback(
    (
      featureCollection: GeoJSON.FeatureCollection,
    ): GeoJSON.FeatureCollection => {
      const pointFeatures = featureCollection.features.filter(
        (feature) => feature.geometry.type === "Point",
      );

      const coordinateGroups = new Map<string, GeoJSON.Feature[]>();

      pointFeatures.forEach((feature) => {
        if (feature.geometry.type === "Point") {
          const [lng, lat] = feature.geometry.coordinates;
          const key = `${lng.toFixed(8)},${lat.toFixed(8)}`;

          if (!coordinateGroups.has(key)) {
            coordinateGroups.set(key, []);
          }
          const group = coordinateGroups.get(key);
          if (group) {
            group.push(feature);
          }
        }
      });

      const processedFeatures = featureCollection.features.map((feature) => {
        if (feature.geometry.type !== "Point") return feature;

        const [lng, lat] = feature.geometry.coordinates;
        const key = `${lng.toFixed(8)},${lat.toFixed(8)}`;
        const group = coordinateGroups.get(key);

        if (!group || group.length === 1) return feature;

        const index = group.findIndex((f) => f === feature);
        const offset = 0.0001;
        const angle = (2 * Math.PI * index) / group.length;
        const offsetLng = lng + offset * Math.cos(angle);
        const offsetLat = lat + offset * Math.sin(angle);

        return {
          ...feature,
          geometry: {
            ...feature.geometry,
            coordinates: [offsetLng, offsetLat],
          },
          properties: {
            ...feature.properties,
            _originalCoordinates: [lng, lat],
            _overlappingCount: group.length,
            _isOffset: group.length > 1,
          },
        };
      });

      return {
        ...featureCollection,
        features: processedFeatures,
      };
    },
    [],
  );

  const processedFileContent = useMemo(() => {
    return processOverlappingPoints(fileContent);
  }, [processOverlappingPoints, fileContent]);

  const pointLayer: LayerProps = useMemo(
    () => ({
      id: "point-layer",
      type: "circle",
      paint: {
        "circle-radius": 5,
        "circle-color": selectedFeatureId
          ? [
              "case",
              ["==", ["get", "_originalId"], selectedFeatureId],
              "#00a340",
              "#3f3f45",
            ]
          : "#3f3f45",
        "circle-stroke-color": "#fff",
        "circle-stroke-width": 1,
      },
      filter: ["==", ["geometry-type"], "Point"],
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

  return (
    <Source
      type={fileType}
      data={processedFileContent}
      cluster={enableClustering}
      promoteId="_originalId">
      {processedFileContent?.features?.some(
        (feature: GeoJSON.Feature) => feature.geometry.type === "Point",
      ) && <Layer {...pointLayer} />}

      {processedFileContent?.features?.some(
        (feature: GeoJSON.Feature) => feature.geometry.type === "LineString",
      ) && <Layer {...lineStringLayer} />}

      {processedFileContent?.features?.some(
        (feature: GeoJSON.Feature) => feature.geometry.type === "Polygon",
      ) && <Layer {...polygonLayer} />}
      <Layer {...clusterLayer} />
      <Layer {...clusterCountLayer} />
    </Source>
  );
};

export { GeoJsonDataSource };
