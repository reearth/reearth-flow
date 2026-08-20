import {
  BoundingSphere,
  Cartesian3,
  HeadingPitchRange,
  Math as CesiumMath,
} from "cesium";

import { gmlGeometriesOf, readCoord } from "./cityGmlGeometryToPrimitives";

export const getFeatureBoundingSphereFromBounds = (gmlGeometries: any[]) => {
  let minX = Infinity;
  let minY = Infinity;
  let minZ = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  let maxZ = -Infinity;
  let found = false;

  for (const geom of gmlGeometries) {
    if (!Array.isArray(geom.polygons)) continue;

    for (const polygon of geom.polygons) {
      for (const coord of polygon.exterior || []) {
        // Either coordinate form: `{ x, y, z }` from the legacy transform,
        // `[lon, lat, z]` from the new-geometry one.
        const position = readCoord(coord);
        if (!position) continue;
        const [x, y, z] = position;

        minX = Math.min(minX, x);
        minY = Math.min(minY, y);
        minZ = Math.min(minZ, z);

        maxX = Math.max(maxX, x);
        maxY = Math.max(maxY, y);
        maxZ = Math.max(maxZ, z);

        found = true;
      }
    }
  }

  if (!found) return null;

  const corners = [
    Cartesian3.fromDegrees(minX, minY, minZ),
    Cartesian3.fromDegrees(minX, minY, maxZ),
    Cartesian3.fromDegrees(minX, maxY, minZ),
    Cartesian3.fromDegrees(minX, maxY, maxZ),
    Cartesian3.fromDegrees(maxX, minY, minZ),
    Cartesian3.fromDegrees(maxX, minY, maxZ),
    Cartesian3.fromDegrees(maxX, maxY, minZ),
    Cartesian3.fromDegrees(maxX, maxY, maxZ),
  ];

  return BoundingSphere.fromPoints(corners);
};

export const zoomToBoundingSphere = (
  geometry: any,
  cesiumViewerRef: any,
  duration: number,
) => {
  // The same normalization the renderer uses, so the camera reaches a
  // new-format CityGML feature too: it has no `gmlGeometries` of its own, its
  // surfaces are read off `coordinates`.
  const gmlGeometries = gmlGeometriesOf(geometry);
  if (!gmlGeometries) return;

  const sphere = getFeatureBoundingSphereFromBounds(gmlGeometries);
  if (!sphere) return;

  const paddedSphere = new BoundingSphere(
    sphere.center,
    Math.max(sphere.radius * 1.2, 10),
  );

  const ce = cesiumViewerRef.current?.cesiumElement;
  if (!ce || ce.isDestroyed()) return;
  ce.camera.flyToBoundingSphere(paddedSphere, {
    duration,
    offset: new HeadingPitchRange(
      0,
      CesiumMath.toRadians(-35),
      paddedSphere.radius * 2.5,
    ),
  });
};
