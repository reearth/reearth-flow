import type { Coordinate } from "../common";

export type PolygonCoordinateRing = Coordinate[];

export type Polygon = {
  type: "Polygon";
  coordinates: PolygonCoordinateRing[];
};
