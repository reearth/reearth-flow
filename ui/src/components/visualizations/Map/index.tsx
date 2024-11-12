import { useEffect, useRef } from "react";
import Maplibre, {
  MapRef,
  NavigationControl,
  ViewState,
} from "react-map-gl/maplibre";

import { SupportedVisualizations } from "../supportedVisualizations";
import "maplibre-gl/dist/maplibre-gl.css";

export type MapMode = Extract<SupportedVisualizations, "3d-map" | "2d-map">;

export const DEFAULT_MAP_TILE_URL =
  "https://basemaps.cartocdn.com/gl/voyager-gl-style/style.json";

export const DEFAULT_MAP_3D_PITCH = 65;

// Centers the map on Japan
export const DEFAULT_MAP_VIEW_STATE: Partial<ViewState> = {
  latitude: 35.68,
  longitude: 139.76,
  zoom: 12,
};

type Props = {
  initialView?: Partial<ViewState>;
  mapMode: MapMode;
};

const Map: React.FC<Props> = ({
  initialView = DEFAULT_MAP_VIEW_STATE,
  mapMode,
}) => {
  const mapRef = useRef<MapRef | null>(null);

  useEffect(() => {
    if (mapRef.current) {
      const mapPitch = mapRef.current.getMap().getPitch();
      if (mapPitch !== 0 && mapMode === "2d-map") {
        mapRef.current.getMap().setPitch(0);
      } else if (mapPitch === 0 && mapMode === "3d-map") {
        mapRef.current.getMap().setPitch(DEFAULT_MAP_3D_PITCH);
      }
    }
  }, [mapMode]);

  return (
    <Maplibre
      ref={mapRef}
      initialViewState={{
        ...initialView,
        pitch: mapMode === "3d-map" ? DEFAULT_MAP_3D_PITCH : 0,
      }}
      style={{ width: "100%", height: "100%" }}
      mapLib={import("maplibre-gl")}
      maxPitch={85}
      mapStyle={DEFAULT_MAP_TILE_URL}>
      <NavigationControl visualizePitch />
    </Maplibre>
  );
};

export { Map };
