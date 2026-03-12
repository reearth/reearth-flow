import {
  BoundingSphere,
  Cartesian3,
  Color,
  ColorGeometryInstanceAttribute,
  ComponentDatatype,
  Geometry,
  GeometryAttribute,
  GeometryAttributes,
  GeometryInstance,
  PerInstanceColorAppearance,
  Primitive,
  PrimitiveCollection,
  PrimitiveType,
  ShowGeometryInstanceAttribute,
} from "cesium";

import type { WorkerOutput } from "./lodGeometryWorker";

/**
 * Build a PrimitiveCollection (fill + outline) from the worker's typed arrays.
 * Runs on the main thread but is fast — no coordinate conversion or triangulation,
 * just wrapping pre-built buffers into Cesium objects.
 */
export function buildLodPrimitiveCollection(
  result: WorkerOutput,
  featureId: string,
): PrimitiveCollection | null {
  if (result.fillPositions.length === 0) return null;

  const fillPrimitive = buildFillPrimitive(
    result.fillPositions,
    result.fillIndices,
    result.fillColors,
    featureId,
  );

  const outlinePrimitive = buildOutlinePrimitive(
    result.outlinePositions,
    result.outlineIndices,
    featureId,
  );

  if (!fillPrimitive) return null;

  const collection = new PrimitiveCollection();
  collection.add(fillPrimitive);
  if (outlinePrimitive) collection.add(outlinePrimitive);
  return collection;
}

/**
 * Group vertices by color and create one GeometryInstance per unique color.
 * This is a lightweight O(V) scan on the main thread — no trig or triangulation.
 */
function buildFillPrimitive(
  positions: Float64Array,
  indices: Uint32Array,
  colors: Float32Array,
  featureId: string,
): Primitive | null {
  if (positions.length === 0 || indices.length === 0) return null;

  const totalVertices = positions.length / 3;

  // Assign each vertex to a color group
  const vertexGroup = new Uint16Array(totalVertices);
  const groupColors: [number, number, number, number][] = [];
  const colorKeyToGroup = new Map<string, number>();

  for (let i = 0; i < totalVertices; i++) {
    const r = colors[i * 4];
    const g = colors[i * 4 + 1];
    const b = colors[i * 4 + 2];
    const a = colors[i * 4 + 3];
    const key = `${r},${g},${b},${a}`;

    let groupIdx = colorKeyToGroup.get(key);
    if (groupIdx === undefined) {
      groupIdx = groupColors.length;
      groupColors.push([r, g, b, a]);
      colorKeyToGroup.set(key, groupIdx);
    }
    vertexGroup[i] = groupIdx;
  }

  // Build per-group geometry
  const instances: GeometryInstance[] = [];

  for (let g = 0; g < groupColors.length; g++) {
    // Find vertices in this group and build old→new index mapping
    const oldToNew = new Map<number, number>();
    const groupPositions: number[] = [];
    let newIdx = 0;

    for (let v = 0; v < totalVertices; v++) {
      if (vertexGroup[v] === g) {
        oldToNew.set(v, newIdx);
        groupPositions.push(
          positions[v * 3],
          positions[v * 3 + 1],
          positions[v * 3 + 2],
        );
        newIdx++;
      }
    }

    if (newIdx === 0) continue;

    // Remap triangle indices
    const groupIndices: number[] = [];
    for (let t = 0; t < indices.length; t += 3) {
      const a = indices[t];
      const b = indices[t + 1];
      const c = indices[t + 2];
      // All 3 vertices of a triangle share the same color (from the same polygon)
      const na = oldToNew.get(a);
      if (na === undefined) continue;
      const nb = oldToNew.get(b);
      const nc = oldToNew.get(c);
      if (nb === undefined || nc === undefined) continue;
      groupIndices.push(na, nb, nc);
    }

    if (groupIndices.length === 0) continue;

    const posArray = new Float64Array(groupPositions);
    const idxArray = new Uint32Array(groupIndices);

    // Compute bounding sphere
    const cartesians: Cartesian3[] = [];
    for (let i = 0; i < newIdx; i++) {
      cartesians.push(
        new Cartesian3(
          posArray[i * 3],
          posArray[i * 3 + 1],
          posArray[i * 3 + 2],
        ),
      );
    }

    const [r, g2, b, a] = groupColors[g];

    instances.push(
      new GeometryInstance({
        id: { featureId, instanceId: `${featureId}_fill_lod_${g}` },
        geometry: (() => {
          const attrs = new GeometryAttributes();
          attrs.position = new GeometryAttribute({
            componentDatatype: ComponentDatatype.DOUBLE,
            componentsPerAttribute: 3,
            values: posArray,
          });
          return new Geometry({
            attributes: attrs,
            indices: idxArray,
            primitiveType: PrimitiveType.TRIANGLES,
            boundingSphere: BoundingSphere.fromPoints(cartesians),
          });
        })(),
        attributes: {
          color: ColorGeometryInstanceAttribute.fromColor(
            new Color(r, g2, b, a),
          ),
          show: new ShowGeometryInstanceAttribute(true),
        },
      }),
    );
  }

  if (instances.length === 0) return null;

  return new Primitive({
    geometryInstances: instances,
    appearance: new PerInstanceColorAppearance({
      flat: true,
      translucent: true,
    }),
    asynchronous: false,
  });
}

function buildOutlinePrimitive(
  positions: Float64Array,
  indices: Uint32Array,
  featureId: string,
): Primitive | null {
  if (positions.length === 0 || indices.length === 0) return null;

  const vertexCount = positions.length / 3;
  const cartesians: Cartesian3[] = [];
  for (let i = 0; i < vertexCount; i++) {
    cartesians.push(
      new Cartesian3(
        positions[i * 3],
        positions[i * 3 + 1],
        positions[i * 3 + 2],
      ),
    );
  }

  return new Primitive({
    geometryInstances: [
      new GeometryInstance({
        id: { featureId, instanceId: `${featureId}_outline_lod` },
        geometry: (() => {
          const attrs = new GeometryAttributes();
          attrs.position = new GeometryAttribute({
            componentDatatype: ComponentDatatype.DOUBLE,
            componentsPerAttribute: 3,
            values: positions,
          });
          return new Geometry({
            attributes: attrs,
            indices,
            primitiveType: PrimitiveType.LINES,
            boundingSphere: BoundingSphere.fromPoints(cartesians),
          });
        })(),
        attributes: {
          color: ColorGeometryInstanceAttribute.fromColor(
            Color.BLACK.withAlpha(0.8),
          ),
          show: new ShowGeometryInstanceAttribute(true),
        },
      }),
    ],
    appearance: new PerInstanceColorAppearance({
      flat: true,
      translucent: true,
    }),
    asynchronous: false,
  });
}
