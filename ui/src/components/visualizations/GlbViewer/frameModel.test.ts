import { BoxGeometry, Mesh, MeshBasicMaterial, Vector3 } from "three";
import { describe, expect, test } from "vitest";

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
