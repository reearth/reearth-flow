import Maplibre from "react-map-gl/maplibre";
import "maplibre-gl/dist/maplibre-gl.css";

const TwoDMap: React.FC = () => {
  return (
    <Maplibre
      initialViewState={{
        longitude: 138.0,
        latitude: 38.0,
        zoom: 4,
      }}
      style={{ width: "100%", height: "100%" }}
      mapStyle="https://basemaps.cartocdn.com/gl/positron-gl-style/style.json"
    />
  );
};

export { TwoDMap };
