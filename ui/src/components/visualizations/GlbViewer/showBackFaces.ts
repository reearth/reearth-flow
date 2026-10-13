import { BackSide, FrontSide, Mesh, MeshBasicMaterial } from "three";
import type { Material, Object3D } from "three";

export const BACK_FACE_COLOR = "#e5484d";

// Unlit, because a back face's normal points away from the light as well as
// the camera, and would otherwise render nearly black. Offset slightly
// behind, so that where a back face lies in the same plane as a front face,
// such as a footprint over a floor, the real face wins rather than flickering.
const backFaceMaterial = new MeshBasicMaterial({
  color: BACK_FACE_COLOR,
  side: BackSide,
  polygonOffset: true,
  polygonOffsetFactor: 1,
  polygonOffsetUnits: 1,
});

const isFrontSided = (material: Material | Material[]) =>
  (Array.isArray(material) ? material : [material]).every(
    (m) => m.side === FrontSide,
  );

/**
 * Draws the back of every face in `object` in a flat colour, where a viewer
 * would otherwise draw nothing.
 *
 * A closed solid whose faces point outward never shows the back of a face from
 * outside. One that does is reporting a defect: a face wound inward, or a gap
 * in a solid that should be closed. Culling hides exactly the faces a user is
 * looking for, so they are shown instead. Meshes already drawn on both sides
 * are left alone.
 *
 * Adds to `object` and returns it.
 */
export const showBackFaces = (object: Object3D): Object3D => {
  const meshes: Mesh[] = [];
  object.traverse((child) => {
    if (child instanceof Mesh && isFrontSided(child.material)) {
      meshes.push(child);
    }
  });

  for (const mesh of meshes) {
    mesh.add(new Mesh(mesh.geometry, backFaceMaterial));
  }
  return object;
};
