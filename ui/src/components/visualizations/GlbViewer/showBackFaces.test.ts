import {
  BackSide,
  BoxGeometry,
  DoubleSide,
  Group,
  Mesh,
  MeshStandardMaterial,
} from "three";
import { describe, expect, test } from "vitest";

import { showBackFaces } from "./showBackFaces";

const meshesIn = (object: Group) => {
  const meshes: Mesh[] = [];
  object.traverse((child) => {
    if (child instanceof Mesh) meshes.push(child);
  });
  return meshes;
};

describe("showBackFaces", () => {
  test("gives each mesh a twin that draws the back of its faces", () => {
    const wall = new Mesh(new BoxGeometry(), new MeshStandardMaterial());
    const model = new Group();
    model.add(wall);

    showBackFaces(model);

    const twin = wall.children[0];
    expect(twin).toBeInstanceOf(Mesh);
    if (!(twin instanceof Mesh)) return;
    expect(twin.geometry).toBe(wall.geometry);
    expect(twin.material.side).toBe(BackSide);
  });

  test("adds one twin per mesh, however deep the meshes sit", () => {
    const model = new Group();
    const storey = new Group();
    storey.add(new Mesh(new BoxGeometry(), new MeshStandardMaterial()));
    storey.add(new Mesh(new BoxGeometry(), new MeshStandardMaterial()));
    model.add(storey);

    showBackFaces(model);

    expect(meshesIn(model)).toHaveLength(4);
  });

  test("leaves a mesh already drawn on both sides alone", () => {
    const model = new Group();
    model.add(
      new Mesh(
        new BoxGeometry(),
        new MeshStandardMaterial({ side: DoubleSide }),
      ),
    );

    showBackFaces(model);

    expect(meshesIn(model)).toHaveLength(1);
  });
});
