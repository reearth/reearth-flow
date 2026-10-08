import { Box3, MathUtils, Sphere, Vector3 } from "three";
import type { Object3D } from "three";

// Seen from above and to one side, so the roof and two walls show at once.
const VIEW_DIRECTION = new Vector3(1, 0.8, 1).normalize();
const MARGIN = 1.4;

export type Framing = {
  /** Where the camera looks from. */
  position: Vector3;
  /** What it looks at, and orbits around. */
  target: Vector3;
  near: number;
  far: number;
  /** How far out the user may zoom while the model stays in view. */
  maxDistance: number;
};

/**
 * Where a camera with a vertical field of view of `fov` degrees should stand
 * to show all of `object`, and the depth range that suits its size.
 *
 * The depth range follows the model, since a building and a whole district
 * differ by orders of magnitude and one fixed range would either clip one or
 * lose depth precision on the other.
 */
export const frameModel = (object: Object3D, fov: number): Framing => {
  const sphere = new Box3()
    .setFromObject(object, true)
    .getBoundingSphere(new Sphere());
  const radius = Math.max(sphere.radius, 1);
  const distance = (radius / Math.sin(MathUtils.degToRad(fov / 2))) * MARGIN;

  return {
    position: sphere.center.clone().addScaledVector(VIEW_DIRECTION, distance),
    target: sphere.center.clone(),
    near: distance / 100,
    far: distance * 100,
    maxDistance: distance * 20,
  };
};
