import { describe, expect, test } from "vitest";

import {
  canContainRaster,
  definitionLabel,
  describeGeometry,
  isNextFormat,
  propertyLabel,
} from "./labels";

describe("describeGeometry", () => {
  test("reads a 2D leaf's type from its discriminant key", () => {
    const result = describeGeometry({
      Euclidean2D: { Point: { frame: { Crs: 4326 }, position: [139.7, 35.6] } },
    });

    expect(result.kind).toBe("2d");
    expect(result.variant).toBe("Point");
    expect(result.definition).toBe("Point2D");
    expect(result.label).toBe("Point (2D)");
    expect(result.value).toEqual({
      frame: { Crs: 4326 },
      position: [139.7, 35.6],
    });
  });

  test("distinguishes the 3D leaf sharing a variant name with a 2D one", () => {
    const result = describeGeometry({
      Euclidean3D: { Polygon: { frame: "Euclidean", exterior: [] } },
    });

    expect(result.kind).toBe("3d");
    expect(result.definition).toBe("Polygon3D");
    expect(result.label).toBe("Polygon (3D)");
  });

  test("resolves 3D-only leaves", () => {
    expect(describeGeometry({ Euclidean3D: { Solid: {} } }).label).toBe(
      "Solid",
    );
    expect(describeGeometry({ Euclidean3D: { PointCloud: {} } }).label).toBe(
      "Point cloud",
    );
  });

  test("treats the bare-string unit variant as an absent geometry", () => {
    const result = describeGeometry("None");

    expect(result.kind).toBe("none");
    expect(result.value).toBeNull();
  });

  test("reports a collection without descending into its members", () => {
    const result = describeGeometry({
      GeometryCollection: { members: ["None"] },
    });

    expect(result.kind).toBe("collection");
    expect(result.label).toBe("Geometry collection");
  });

  test("returns unknown for a legacy-format geometry", () => {
    const result = describeGeometry({
      epsg: 4326,
      value: { flowGeometry2D: { point: { x: 1, y: 2 } } },
    });

    expect(result.kind).toBe("unknown");
  });

  test("returns unknown rather than throwing on an unrecognised variant", () => {
    expect(describeGeometry({ Euclidean2D: { Hyperbola: {} } }).kind).toBe(
      "2d",
    );
    expect(describeGeometry(null).kind).toBe("unknown");
    expect(describeGeometry({ a: 1, b: 2 }).kind).toBe("unknown");
  });
});

describe("isNextFormat", () => {
  test("accepts every top-level new-format variant", () => {
    expect(isNextFormat({ geometry: "None" })).toBe(true);
    expect(isNextFormat({ geometry: { Euclidean2D: {} } })).toBe(true);
    expect(isNextFormat({ geometry: { Euclidean3D: {} } })).toBe(true);
    expect(isNextFormat({ geometry: { GeometryCollection: {} } })).toBe(true);
  });

  test("rejects the legacy wrapper and missing geometry", () => {
    expect(isNextFormat({ geometry: { epsg: 4326, value: "None" } })).toBe(
      false,
    );
    expect(isNextFormat({ geometry: { value: {} } })).toBe(false);
    expect(isNextFormat({})).toBe(false);
  });
});

describe("labels", () => {
  test("prefers the schema title over the raw definition name", () => {
    expect(definitionLabel("TriangularMesh2D")).toBe("Triangle mesh (2D)");
    expect(definitionLabel("RasterData")).toBe("Raster data");
  });

  test("falls back to the raw name for definitions the schema does not title", () => {
    expect(definitionLabel("NotADefinition")).toBe("NotADefinition");
    expect(definitionLabel(null)).toBe("");
  });

  test("labels properties per owning definition", () => {
    expect(propertyLabel("Polygon3D", "exterior")).toBe("Exterior ring");
    expect(propertyLabel("RasterData", "bytes")).toBe("Image bytes");
    expect(propertyLabel("Polygon3D", "nope")).toBe("nope");
  });
});

describe("canContainRaster", () => {
  test("includes every path that reaches a texture", () => {
    for (const definition of [
      "Geometry",
      "Euclidean3DGeometry",
      "Solid",
      "Shell",
      "Csg",
      "Collection2D",
      "PolygonMesh3DData",
      "Appearance",
      "Texture",
    ]) {
      expect(canContainRaster(definition)).toBe(true);
    }
  });

  test("excludes the bulky leaves a raster walk should prune", () => {
    for (const definition of [
      "Point2D",
      "LineString3D",
      "PointCloud",
      "Segment",
      "FaceUv",
      "TexMatrix",
      "CoordinateFrame",
    ]) {
      expect(canContainRaster(definition)).toBe(false);
    }
  });
});
