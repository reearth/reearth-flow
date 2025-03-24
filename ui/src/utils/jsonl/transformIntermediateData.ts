export function intermediateDataTransform(parsedData: any) {
  if (parsedData.geometry) {
    const is2D = "flowGeometry2D" in parsedData.geometry.value;
    const is3D = "flowGeometry3D" in parsedData.geometry.value;
    const isUnknown = !is2D && !is3D;

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
      console.warn(
        "3D geometry detected, but 3D viewer is not supported yet. Displaying raw data.",
      );
      return parsedData;
    }
  }
  return parsedData;
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
  return geometry;
}
