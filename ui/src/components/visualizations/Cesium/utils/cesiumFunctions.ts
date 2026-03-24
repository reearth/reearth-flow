import {
  BoundingSphere,
  Cartesian3,
  HeadingPitchRange,
  Math as CesiumMath,
} from "cesium";

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
        if (
          coord &&
          typeof coord.x === "number" &&
          typeof coord.y === "number"
        ) {
          const z = typeof coord.z === "number" ? coord.z : 0;

          minX = Math.min(minX, coord.x);
          minY = Math.min(minY, coord.y);
          minZ = Math.min(minZ, z);

          maxX = Math.max(maxX, coord.x);
          maxY = Math.max(maxY, coord.y);
          maxZ = Math.max(maxZ, z);

          found = true;
        }
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
  const gmlGeometries =
    geometry.gmlGeometries || geometry.value?.cityGmlGeometry?.gmlGeometries;
  const positions: Cartesian3[] = [];
  if (!Array.isArray(gmlGeometries)) return;
  for (const geom of gmlGeometries) {
    if (!Array.isArray(geom.polygons)) continue;

    for (const polygon of geom.polygons) {
      const rings = [
        ...(polygon.exterior ? [polygon.exterior] : []),
        ...(Array.isArray(polygon.interiors) ? polygon.interiors : []),
      ];

      for (const ring of rings) {
        if (!Array.isArray(ring)) continue;
        for (const coord of ring || []) {
          if (
            coord &&
            typeof coord.x === "number" &&
            typeof coord.y === "number"
          ) {
            positions.push(
              Cartesian3.fromDegrees(
                coord.x,
                coord.y,
                typeof coord.z === "number" ? coord.z : 0,
              ),
            );
          }
        }
      }
    }
  }

  if (positions.length === 0) return;

  const sphere = getFeatureBoundingSphereFromBounds(gmlGeometries);
  if (!sphere) return;

  const paddedSphere = new BoundingSphere(
    sphere.center,
    Math.max(sphere.radius * 1.2, 10),
  );

  cesiumViewerRef.current?.cesiumElement.camera.flyToBoundingSphere(
    paddedSphere,
    {
      duration,
      offset: new HeadingPitchRange(
        0,
        CesiumMath.toRadians(-35),
        paddedSphere.radius * 2.5,
      ),
    },
  );
};
