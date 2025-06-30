import { useMemo } from "react";
import { Source, Marker, Layer, LayerProps } from "react-map-gl/maplibre";

type Props = {
  fileType: "geojson";
  fileContent: GeoJSON.FeatureCollection<
    GeoJSON.Geometry,
    GeoJSON.GeoJsonProperties
  >;
  onSelectedFeature: (value: any) => void;
};

const GeoJsonDataSource: React.FC<Props> = ({
  fileType,
  fileContent,
  onSelectedFeature,
}) => {
  const polygonLayer: LayerProps = useMemo(
    () => ({
      id: "polygon-layer",
      type: "fill",
      paint: {
        "fill-color": "#3f3f45",
        "fill-opacity": 0.8,
      },
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
    }),
    [],
  );

  const pointerMarker = useMemo(
    () => ({
      color: "#3f3f45",
      onClick: (e: any, feature: any) => {
        e.originalEvent.stopPropagation();
        onSelectedFeature(feature);
      },
    }),
    [onSelectedFeature],
  );
  return (
    <Source type={fileType} data={fileContent}>
      {fileContent?.features?.some(
        (feature: GeoJSON.Feature) => feature.geometry.type === "LineString",
      ) && (
        <Layer
          {...lineStringLayer}
          filter={["==", ["geometry-type"], "LineString"]}
        />
      )}

      {fileContent?.features?.some(
        (feature: GeoJSON.Feature) => feature.geometry.type === "Polygon",
      ) && (
        <Layer
          {...polygonLayer}
          filter={["==", ["geometry-type"], "Polygon"]}
        />
      )}

      {fileContent?.features?.map((feature: any, i: number) => {
        if (feature.geometry?.type === "MultiPoint") {
          return feature.geometry.coordinates.map((coords: any, j: number) => (
            <Marker
              key={`${i}-${j}`}
              longitude={coords[0]}
              latitude={coords[1]}
              color={pointerMarker.color}
              onClick={(e) => pointerMarker.onClick(e, feature)}
            />
          ));
        } else if (feature.geometry?.type === "Point") {
          const coords = feature.geometry.coordinates;
          return (
            <Marker
              key={i}
              color="#3f3f45"
              longitude={coords[0]}
              latitude={coords[1]}
              onClick={(e) => {
                e.originalEvent.stopPropagation();
                onSelectedFeature(feature);
              }}
            />
          );
        } else return null;
      })}
    </Source>
  );
};

export { GeoJsonDataSource };
