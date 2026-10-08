import { BoxGeometry, Mesh, MeshBasicMaterial, Vector3 } from "three";
import { describe, expect, test } from "vitest";

import type { Framing } from "./frameModel";
import { frameModel } from "./frameModel";

const building = (width: number, height: number) => {
  const mesh = new Mesh(
    new BoxGeometry(width, height, width),
    new MeshBasicMaterial(),
  );
  mesh.position.set(0, height / 2, 0);
  mesh.updateMatrixWorld(true);
  return mesh;
};

describe("frameModel", () => {
  test("looks at the model's centre", () => {
    const { target } = frameModel(building(10, 20), 50);

    expect(target.distanceTo(new Vector3(0, 10, 0))).toBeCloseTo(0, 6);
  });

  test("stands far enough back to see all of it, from above", () => {
    const { position, target } = frameModel(building(10, 20), 50);
    const radius = new Vector3(5, 10, 5).length();

    expect(position.distanceTo(target)).toBeGreaterThan(radius * 2);
    expect(position.y).toBeGreaterThan(target.y);
  });

  test("stands further back in a view narrower than it is tall", () => {
    const model = building(10, 20);
    const square = frameModel(model, 50, 1);
    const wide = frameModel(model, 50, 2);
    const narrow = frameModel(model, 50, 0.5);
    const distance = (f: Framing) => f.position.distanceTo(f.target);

    // Wider than tall, the height still limits the view.
    expect(distance(wide)).toBeCloseTo(distance(square), 6);
    // Half as wide as tall, the sides limit it: the model's bounding sphere
    // must fit the horizontal half-angle.
    const radius = new Vector3(5, 10, 5).length();
    const halfWidth = Math.atan(Math.tan((25 * Math.PI) / 180) * 0.5);
    expect(distance(narrow)).toBeGreaterThan(radius / Math.sin(halfWidth));
  });

  test("keeps the whole zoom range inside the depth range", () => {
    const { near, far, maxDistance } = frameModel(building(10, 20), 50);

    expect(near).toBeGreaterThan(0);
    expect(maxDistance).toBeLessThan(far);
  });

  test("scales with the model", () => {
    const house = frameModel(building(10, 10), 50);
    const tower = frameModel(building(100, 100), 50);

    expect(tower.far / house.far).toBeCloseTo(10, 6);
    expect(tower.near / house.near).toBeCloseTo(10, 6);
  });
});
