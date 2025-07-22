import { useMemo } from "react";
import { Source, Layer, LayerProps } from "react-map-gl/maplibre";

type Props = {
  fileType: "geojson";
  fileContent: GeoJSON.FeatureCollection<
    GeoJSON.Geometry,
    GeoJSON.GeoJsonProperties
  >;
  enableClustering?: boolean;
};

const GeoJsonDataSource: React.FC<Props> = ({
  fileType,
  fileContent,
  enableClustering,
}) => {
  const pointLayer: LayerProps = useMemo(
    () => ({
      id: "point-layer",
      type: "circle",
      paint: {
        "circle-radius": 5,
        "circle-color": "#3f3f45",
        "circle-stroke-color": "#fff",
        "circle-stroke-width": 1,
      },
      filter: ["==", ["geometry-type"], "Point"],
    }),
    [],
  );

  const lineStringLayer: LayerProps = useMemo(
    () => ({
      id: "line-layer",
      type: "line",
      paint: {
        "line-color": "#3f3f45",
        "line-width": 2,
      },
      filter: ["==", ["geometry-type"], "LineString"],
    }),
    [],
  );

  const polygonLayer: LayerProps = useMemo(
    () => ({
      id: "polygon-layer",
      type: "fill",
      paint: {
        "fill-color": "#3f3f45",
        "fill-opacity": 0.8,
      },
      filter: ["==", ["geometry-type"], "Polygon"],
    }),
    [],
  );

  const clusterLayer: LayerProps = {
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
  };

  const clusterCountLayer: LayerProps = {
    id: "cluster-count",
    type: "symbol",
    filter: ["has", "point_count"],
    layout: {
      "text-field": "{point_count_abbreviated}",
      "text-size": 12,
    },
  };

  return (
    <Source type={fileType} data={fileContent} cluster={enableClustering}>
      {fileContent?.features?.some(
        (feature: GeoJSON.Feature) => feature.geometry.type === "Point",
      ) && <Layer {...pointLayer} />}

      {fileContent?.features?.some(
        (feature: GeoJSON.Feature) => feature.geometry.type === "LineString",
      ) && <Layer {...lineStringLayer} />}

      {fileContent?.features?.some(
        (feature: GeoJSON.Feature) => feature.geometry.type === "Polygon",
      ) && <Layer {...polygonLayer} />}

      <Layer {...clusterLayer} />
      <Layer {...clusterCountLayer} />
    </Source>
  );
};

export { GeoJsonDataSource };
