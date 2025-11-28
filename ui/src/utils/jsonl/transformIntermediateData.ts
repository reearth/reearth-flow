export function intermediateDataTransform(parsedData: any) {
  if (parsedData.geometry) {
    if (parsedData.geometry.value === "none") {
      return {
        id: parsedData.id,
        type: "Feature",
        properties: { ...parsedData.attributes },
      };
    }

    const is2D = "flowGeometry2D" in parsedData.geometry.value;
    const is3D = "flowGeometry3D" in parsedData.geometry.value;
    const isCityGml = "cityGmlGeometry" in parsedData.geometry.value;
    const isUnknown = !is2D && !is3D && !isCityGml;

    if (isUnknown) {
      console.warn("Unknown geometry type detected. Displaying raw data.");
      return parsedData;
    }

    if (is2D) {
      return {
        id: parsedData.id,
        type: "Feature",
        properties: { ...parsedData.attributes },
        geometry: handle2DGeometry(parsedData.geometry.value.flowGeometry2D),
      };
    }

    if (is3D) {
      return {
        id: parsedData.id,
        type: "Feature",
        properties: { ...parsedData.attributes },
        geometry: handle3DGeometry(parsedData.geometry.value.flowGeometry3D),
      };
    }

    if (isCityGml) {
      return {
        id: parsedData.id,
        type: "Feature",
        properties: { ...parsedData.attributes },
        geometry: handleCityGmlGeometry(
          parsedData.geometry.value.cityGmlGeometry,
        ),
      };
    }
  }
  return parsedData;
}

function buildClosedRing(coords: [number, number][]): [number, number][] {
  if (coords.length === 0) return coords;

  const firstPoint = coords[0];
  const lastPoint = coords[coords.length - 1];

  return firstPoint[0] !== lastPoint[0] || firstPoint[1] !== lastPoint[1]
    ? [...coords, firstPoint]
    : coords;
}

function handle2DGeometry(geometry: any) {
  if ("point" in geometry) {
    const coordinateValues = Object.values(geometry.point);
    return {
      type: "Point",
      coordinates: [coordinateValues[0], coordinateValues[1]],
    };
  }
  if ("polygon" in geometry) {
    const coordinates = [
      [...geometry.polygon.exterior.map((point: any) => [point.x, point.y])],
    ];
    if (geometry.polygon.interiors && geometry.polygon.interiors.length) {
      coordinates.push([
        ...geometry.polygon.interiors.map((interior: any) => {
          return interior.map((point: any) => [point.x, point.y]);
        }),
      ]);
    }
    return {
      type: "Polygon",
      coordinates,
    };
  }
  if ("lineString" in geometry) {
    const coordinates = geometry.lineString.map((point: any) => {
      const c = Object.values(point);
      return [c[0], c[1]];
    });
    return {
      type: "LineString",
      coordinates,
    };
  }
  if ("multiPoint" in geometry) {
    const coordinates = geometry.multiPoint.map((point: any) => {
      const c = Object.values(point);
      return [c[0], c[1]];
    });
    return {
      type: "MultiPoint",
      coordinates,
    };
  }
  if ("multiPolygon" in geometry) {
    const coordinates = geometry.multiPolygon.map((polygon: any) => {
      const polyCoords = [
        [...polygon.exterior.map((point: any) => [point.x, point.y])],
      ];
      if (polygon.interiors) {
        polyCoords.push(...polygon.interiors);
      }
      return polyCoords;
    });
    return {
      type: "MultiPolygon",
      coordinates,
    };
  }
  if ("multiLineString" in geometry) {
    const coordinates = geometry.multiLineString.map((lineString: any) => {
      return lineString.map((point: any) => {
        const c = Object.values(point);
        return [c[0], c[1]];
      });
    });
    return {
      type: "MultiLineString",
      coordinates,
    };
  }

  if ("triangle" in geometry) {
    const coords = geometry.triangle.map((point: any) => [point.x, point.y]);
    return {
      type: "Polygon",
      coordinates: [buildClosedRing(coords)],
    };
  }

  if ("rect" in geometry) {
    const { min, max } = geometry.rect;
    const coordinates = [
      buildClosedRing([
        [min.x, min.y],
        [max.x, min.y],
        [max.x, max.y],
        [min.x, max.y],
      ]),
    ];
    return {
      type: "Polygon",
      coordinates,
    };
  }
  return geometry;
}

function handleCityGmlGeometry(geometry: any) {
  const result: any = {
    type: "CityGmlGeometry",
  };

  // Dynamically add only properties that exist in the data
  Object.keys(geometry).forEach((key) => {
    result[key] = geometry[key];
  });

  return result;
}

function handle3DGeometry(geometry: any) {
  if ("point" in geometry) {
    const coordinateValues = Object.values(geometry.point);
    return {
      type: "Point",
      coordinates: [
        coordinateValues[0],
        coordinateValues[1],
        coordinateValues[2],
      ],
    };
  }
  if ("polygon" in geometry) {
    const coordinates = [
      [
        ...geometry.polygon.exterior.map((point: any) => [
          point.x,
          point.y,
          point.z || 0,
        ]),
      ],
    ];
    if (geometry.polygon.interiors && geometry.polygon.interiors.length) {
      coordinates.push([
        ...geometry.polygon.interiors.map((interior: any) => {
          return interior.map((point: any) => [point.x, point.y, point.z || 0]);
        }),
      ]);
    }
    return {
      type: "Polygon",
      coordinates,
    };
  }
  if ("lineString" in geometry) {
    const coordinates = geometry.lineString.map((point: any) => {
      const c = Object.values(point);
      return [c[0], c[1], c[2] || 0];
    });
    return {
      type: "LineString",
      coordinates,
    };
  }
  if ("multiPoint" in geometry) {
    const coordinates = geometry.multiPoint.map((point: any) => {
      const c = Object.values(point);
      return [c[0], c[1], c[2] || 0];
    });
    return {
      type: "MultiPoint",
      coordinates,
    };
  }
  if ("multiPolygon" in geometry) {
    const coordinates = geometry.multiPolygon.map((polygon: any) => {
      const polyCoords = [
        [
          ...polygon.exterior.map((point: any) => [
            point.x,
            point.y,
            point.z || 0,
          ]),
        ],
      ];
      if (polygon.interiors) {
        polyCoords.push(...polygon.interiors);
      }
      return polyCoords;
    });
    return {
      type: "MultiPolygon",
      coordinates,
    };
  }
  if ("multiLineString" in geometry) {
    const coordinates = geometry.multiLineString.map((lineString: any) => {
      return lineString.map((point: any) => {
        const c = Object.values(point);
        return [c[0], c[1], c[2] || 0];
      });
    });
    return {
      type: "MultiLineString",
      coordinates,
    };
  }

  if ("solid" in geometry) {
    // Extract boundary surface from solid
    const boundarySurface = geometry.solid.boundary_surface;

    // Handle Faces variant
    if ("Faces" in boundarySurface) {
      const faces = boundarySurface.Faces;

      // Convert each face to a polygon and create a MultiPolygon
      const coordinates = faces.map((face: any) => {
        // Each face is an array of coordinates
        const faceCoords = face.map((coord: any) => [
          coord.x,
          coord.y,
          coord.z || 0,
        ]);

        // Ensure the face is closed (first point === last point)
        if (faceCoords.length > 0) {
          const firstPoint = faceCoords[0];
          const lastPoint = faceCoords[faceCoords.length - 1];

          if (
            firstPoint[0] !== lastPoint[0] ||
            firstPoint[1] !== lastPoint[1] ||
            firstPoint[2] !== lastPoint[2]
          ) {
            faceCoords.push(firstPoint);
          }
        }

        // Return as a polygon with a single exterior ring (no holes)
        return [faceCoords];
      });

      return {
        type: "MultiPolygon",
        coordinates,
      };
    }

    // Handle TriangularMesh variant
    if ("TriangularMesh" in boundarySurface) {
      const mesh = boundarySurface.TriangularMesh;
      const vertices = mesh.vertices;
      const triangles = mesh.triangles;

      // Convert each triangle to a polygon
      const coordinates = triangles.map((triangle: number[]) => {
        const triangleCoords = [
          [
            vertices[triangle[0]].x,
            vertices[triangle[0]].y,
            vertices[triangle[0]].z || 0,
          ],
          [
            vertices[triangle[1]].x,
            vertices[triangle[1]].y,
            vertices[triangle[1]].z || 0,
          ],
          [
            vertices[triangle[2]].x,
            vertices[triangle[2]].y,
            vertices[triangle[2]].z || 0,
          ],
          [
            vertices[triangle[0]].x,
            vertices[triangle[0]].y,
            vertices[triangle[0]].z || 0,
          ], // Close the triangle
        ];

        return [triangleCoords];
      });

      return {
        type: "MultiPolygon",
        coordinates,
      };
    }
  }

  // For any 3D geometry types not handled above, return the raw structure
  return {
    type: "FlowGeometry3D",
    ...geometry,
  };
}
