import {
  BoxGeometry,
  Color,
  Group,
  Mesh,
  MeshStandardMaterial,
  Texture,
} from "three";
import { describe, expect, test } from "vitest";

import { FEATURE_COLOR } from "../featureColors";

import { tintDefaultMaterials } from "./tintDefaultMaterials";

const mesh = (material: MeshStandardMaterial) =>
  new Mesh(new BoxGeometry(), material);

describe("tintDefaultMaterials", () => {
  test("colours the engine's default gray, without changing the loaded material", () => {
    const gray = new MeshStandardMaterial({ color: new Color(0.7, 0.7, 0.7) });
    const root = new Group().add(mesh(gray), mesh(gray));

    tintDefaultMaterials(root);

    const [a, b] = root.children as Mesh[];
    expect((a.material as MeshStandardMaterial).color.getHex()).toBe(
      new Color(FEATURE_COLOR).getHex(),
    );
    // One replacement per material, and the original is untouched.
    expect(a.material).toBe(b.material);
    expect(gray.color.r).toBeCloseTo(0.7);
  });

  test("leaves the data's own colours and textures alone", () => {
    const red = new MeshStandardMaterial({ color: new Color(1, 0, 0) });
    const textured = new MeshStandardMaterial({
      color: new Color(0.7, 0.7, 0.7),
      map: new Texture(),
    });
    const root = new Group().add(mesh(red), mesh(textured));

    tintDefaultMaterials(root);

    const [a, b] = root.children as Mesh[];
    expect(a.material).toBe(red);
    expect(b.material).toBe(textured);
  });
});
