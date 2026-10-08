import { Box3, Group, Vector3 } from "three";
import type { Object3D } from "three";

// A root node further than this from the origin is taken to be placed on the
// globe rather than in model space. The Earth's radius is about 6,371 km.
const GEOREFERENCED_DISTANCE = 1_000_000;

const Y_UP = new Vector3(0, 1, 0);

/**
 * Stands a georeferenced glTF scene upright at the origin, resting on y = 0.
 *
 * The engine writes a rendered row the way 3D Tiles content is written: its
 * vertices are local to an origin, and the root node's translation puts that
 * origin on the globe, in Earth-centred coordinates with Y up. Shown as is, a
 * model sits thousands of kilometres from the camera, tilted by its latitude
 * and longitude. Pointing that translation's direction, which is "up" where
 * the model stands, along +Y and dropping the translation itself brings it
 * home. Its heading around that axis is left as it comes.
 *
 * A scene with no georeferenced root is only moved to rest on the grid.
 */
export const placeUpright = (scene: Object3D): Object3D => {
  const model = scene.clone(true);
  const anchor = model.children.find(
    (child) => child.position.length() > GEOREFERENCED_DISTANCE,
  );

  const upright = new Group();
  upright.add(model);

  if (anchor) {
    const origin = anchor.position.clone();
    for (const child of model.children) child.position.sub(origin);
    upright.quaternion.setFromUnitVectors(origin.normalize(), Y_UP);
  }

  const placed = new Group();
  placed.add(upright);
  placed.updateMatrixWorld(true);

  // Measured from the vertices: the default boxes each mesh's own bounds
  // after rotation, which overstates a tilted model and lifts it off the grid.
  const bounds = new Box3().setFromObject(placed, true);
  if (!bounds.isEmpty()) {
    const center = bounds.getCenter(new Vector3());
    placed.position.set(-center.x, -bounds.min.y, -center.z);
    placed.updateMatrixWorld(true);
  }

  return placed;
};
