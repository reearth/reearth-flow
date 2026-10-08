import { Color, Mesh, MeshStandardMaterial } from "three";
import type { Material, Object3D } from "three";

import { FEATURE_COLOR } from "../featureColors";

/**
 * The flat gray the engine gives a face bound to no material
 * (`DEFAULT_MATERIAL` in the 3D Tiles writer's `primitive.rs`).
 */
const ENGINE_DEFAULT_GRAY = 0.7;

const isEngineDefault = (material: Material) =>
  material instanceof MeshStandardMaterial &&
  !material.map &&
  [material.color.r, material.color.g, material.color.b].every(
    (channel) => Math.abs(channel - ENGINE_DEFAULT_GRAY) < 1e-3,
  );

/**
 * Draws the faces that carry no colour of their own in the views' feature
 * colour, and leaves the data's own appearance alone. Materials are replaced,
 * not changed, since the loader shares them between loads of the same file.
 */
export const tintDefaultMaterials = (root: Object3D) => {
  const tinted = new Map<Material, Material>();
  const tint = (material: Material) => {
    if (!isEngineDefault(material)) return material;
    let replacement = tinted.get(material);
    if (!replacement) {
      const copy = (material as MeshStandardMaterial).clone();
      copy.color = new Color(FEATURE_COLOR);
      replacement = copy;
      tinted.set(material, replacement);
    }
    return replacement;
  };

  root.traverse((child) => {
    if (!(child instanceof Mesh)) return;
    child.material = Array.isArray(child.material)
      ? child.material.map(tint)
      : tint(child.material);
  });
  return root;
};
