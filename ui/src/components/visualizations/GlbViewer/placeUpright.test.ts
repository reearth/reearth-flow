import {
  Box3,
  BufferGeometry,
  Float32BufferAttribute,
  Group,
  Mesh,
  Vector3,
} from "three";
import { describe, expect, test } from "vitest";

import { placeUpright } from "./placeUpright";

// The root translation of a building the engine rendered near Toyohashi, in
// Earth-centred coordinates with Y up.
const ORIGIN = new Vector3(-3865606.98, 3615071.87, -3547188.83);

/**
 * A 30 m pillar standing on the ground where `ORIGIN` is, in the glTF's local
 * frame: it runs along "up" there, which is the origin's own direction.
 */
const pillarScene = () => {
  const up = ORIGIN.clone().normalize();
  const side = new Vector3(0, 1, 0).cross(up).normalize();
  const vertices = [
    side.clone().multiplyScalar(-2),
    side.clone().multiplyScalar(2),
    up.clone().multiplyScalar(30),
  ].flatMap((v) => [v.x, v.y, v.z]);

  const geometry = new BufferGeometry();
  geometry.setAttribute("position", new Float32BufferAttribute(vertices, 3));
  const node = new Mesh(geometry);
  node.position.copy(ORIGIN);

  const scene = new Group();
  scene.add(node);
  return scene;
};

const boundsOf = (object: ReturnType<typeof placeUpright>) =>
  new Box3().setFromObject(object, true);

describe("placeUpright", () => {
  test("brings a georeferenced model to the origin", () => {
    const bounds = boundsOf(placeUpright(pillarScene()));

    expect(bounds.getCenter(new Vector3()).x).toBeCloseTo(0, 3);
    expect(bounds.getCenter(new Vector3()).z).toBeCloseTo(0, 3);
  });

  test("stands it up along +Y, resting on the grid", () => {
    const bounds = boundsOf(placeUpright(pillarScene()));

    expect(bounds.min.y).toBeCloseTo(0, 3);
    expect(bounds.max.y).toBeCloseTo(30, 3);
  });

  test("leaves the source scene untouched", () => {
    const scene = pillarScene();
    placeUpright(scene);

    expect(scene.children[0].position.equals(ORIGIN)).toBe(true);
  });

  test("only rests a model-space scene on the grid", () => {
    const geometry = new BufferGeometry();
    geometry.setAttribute(
      "position",
      new Float32BufferAttribute([0, -5, 0, 1, 5, 0, 0, 5, 1], 3),
    );
    const scene = new Group();
    scene.add(new Mesh(geometry));

    const bounds = boundsOf(placeUpright(scene));

    expect(bounds.min.y).toBeCloseTo(0, 6);
    expect(bounds.max.y).toBeCloseTo(10, 6);
  });
});
