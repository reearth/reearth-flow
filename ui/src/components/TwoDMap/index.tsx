import maplibregl, {
  RasterSourceSpecification,
  SourceSpecification,
} from "maplibre-gl";
import { Protocol, PMTiles } from "pmtiles";
import React, { useRef, useEffect, useState } from "react";
import "maplibre-gl/dist/maplibre-gl.css";

const PMTILES_URL = "PMTilesのパス";

const sourceSetting: Record<string, SourceSpecification> = {
  osm: {
    type: "raster",
    tiles: ["https://tile.openstreetmap.org/{z}/{x}/{y}.png"],
    tileSize: 256,
    attribution:
      '© <a href="https://openstreetmap.org">OpenStreetMap</a> contributors',
  } as RasterSourceSpecification,
};

// 背景地図の設定
const baseLayer: maplibregl.LayerSpecification[] = [
  {
    id: "osm-tiles",
    type: "raster",
    source: "osm",
    minzoom: 0,
    maxzoom: 19,
  },
];

// PMTilesから取得して表示、非表示を制御するレイヤ達
const otherLayers: maplibregl.LayerSpecification[] = [
  {
    id: "gyousei-kukaku",
    source: "gyousei-kukaku",
    "source-layer": "N0323_230101",
    type: "fill",
    paint: {
      "fill-color": "#00ffff",
      "fill-opacity": 0.4,
      "fill-outline-color": "#ff0000",
    },
    layout: {
      visibility: "none", // 最初から表示する場合は "visible"に
    },
  },
  {
    id: "hazard-map",
    source: "fukuoka-hazardmap",
    "source-layer": "fukuokahazardmap",
    type: "fill",
    paint: {
      "fill-color": "#ff0000",
      "fill-opacity": 0.4,
      "fill-outline-color": "#0000ff",
    },
    layout: {
      visibility: "none", // 最初から表示する場合は "visible"に
    },
  },
];

type Props = {
  aProp?: string;
};

const TwoDMap: React.FC<Props> = () => {
  const mapContainer = useRef(null);
  const map = useRef<any>(null);
  const [mapState, setMapState] = useState({
    lat: 33.5676,
    lon: 130.4102,
    zoom: 9,
  });

  useEffect(() => {
    if (map.current) return; // initialize map only once

    const layersSetting = [...baseLayer, ...otherLayers];

    const protocol = new Protocol();
    maplibregl.addProtocol("pmtiles", protocol.tile);
    const p = new PMTiles(PMTILES_URL);
    protocol.add(p);

    map.current = new maplibregl.Map({
      container: mapContainer.current!, //eslint-disable-line
      center: [mapState.lon, mapState.lat],
      zoom: mapState.zoom,
      style: {
        version: 8,
        sources: sourceSetting,
        layers: layersSetting,
      },
    });

    map.current.on("move", () => {
      setMapState({
        lat: Number(map.current.getCenter().lat.toFixed(4)),
        lon: Number(map.current.getCenter().lng.toFixed(4)),
        zoom: Number(map.current.getZoom().toFixed(2)),
      });
    });
  }, [mapState]);

  return (
    <div
      ref={mapContainer}
      className="size-full"
      style={{ height: "70vh", width: "100%" }}
    />
  );
};

export { TwoDMap };
