export const pointGeoJsonData = {
  type: "FeatureCollection",
  features: [
    {
      type: "Feature",
      properties: {
        name: "Shinjuku",
        population: 346000,
        ward: "Shinjuku-ku",
      },
      geometry: {
        type: "Point",
        coordinates: [139.7051, 35.6938],
      },
    },
    {
      type: "Feature",
      properties: {
        name: "Shibuya",
        population: 228000,
        ward: "Shibuya-ku",
      },
      geometry: {
        type: "Point",
        coordinates: [139.7015, 35.658],
      },
    },
    {
      type: "Feature",
      properties: {
        name: "Akihabara",
        population: 99000,
        ward: "Chiyoda-ku",
      },
      geometry: {
        type: "Point",
        coordinates: [139.7714, 35.699],
      },
    },
  ],
};

export const polygonGeoJsonData = {
  type: "FeatureCollection",
  features: [
    {
      type: "Feature",
      properties: {
        name: "Shinjuku Ward",
        area: "18.23 km²",
        population: 346000,
      },
      geometry: {
        type: "Polygon",
        coordinates: [
          [
            [139.6917, 35.7056],
            [139.7245, 35.7056],
            [139.7245, 35.683],
            [139.6917, 35.683],
            [139.6917, 35.7056],
          ],
        ],
      },
    },
    {
      type: "Feature",
      properties: {
        name: "Shibuya Ward",
        area: "15.11 km²",
        population: 228000,
      },
      geometry: {
        type: "Polygon",
        coordinates: [
          [
            [139.685, 35.67],
            [139.715, 35.67],
            [139.715, 35.65],
            [139.685, 35.65],
            [139.685, 35.67],
          ],
        ],
      },
    },
    {
      type: "Feature",
      properties: {
        name: "Chiyoda Ward",
        area: "11.66 km²",
        population: 66000,
      },
      geometry: {
        type: "Polygon",
        coordinates: [
          [
            [139.75, 35.705],
            [139.775, 35.705],
            [139.775, 35.68],
            [139.75, 35.68],
            [139.75, 35.705],
          ],
        ],
      },
    },
    {
      type: "Feature",
      properties: {
        name: "Minato Ward",
        area: "20.37 km²",
        population: 257000,
      },
      geometry: {
        type: "Polygon",
        coordinates: [
          [
            [139.73, 35.67],
            [139.76, 35.67],
            [139.76, 35.64],
            [139.73, 35.64],
            [139.73, 35.67],
          ],
        ],
      },
    },
  ],
};

export const lineStringGeoJsonData = {
  type: "FeatureCollection",
  features: [
    {
      type: "Feature",
      properties: {
        name: "Tachikawa to Tokyo Station",
        transit: "JR Chuo Line",
        distance: "40.2 km",
      },
      geometry: {
        type: "LineString",
        coordinates: [
          [139.4137, 35.6987],
          [139.466, 35.7033],
          [139.5326, 35.7027],
          [139.5795, 35.7031],
          [139.6389, 35.7038],
          [139.6994, 35.7021],
          [139.7671, 35.6812],
        ],
      },
    },
  ],
};

export const mixtureGeoJsonData = {
  type: "FeatureCollection",
  features: [
    {
      type: "Feature",
      geometry: {
        type: "Point",
        coordinates: [139.6917, 35.6895],
      },
      properties: {
        name: "Tokyo Center",
      },
    },
    {
      type: "Feature",
      geometry: {
        type: "MultiPoint",
        coordinates: [
          [139.702, 35.6896],
          [139.7753, 35.71],
        ],
      },
      properties: {
        name: "Multiple Landmarks",
      },
    },
    {
      type: "Feature",
      geometry: {
        type: "LineString",
        coordinates: [
          [139.767, 35.6812],
          [139.7004, 35.6895],
        ],
      },
      properties: {
        name: "Tokyo Line A",
      },
    },
    {
      type: "Feature",
      geometry: {
        type: "MultiLineString",
        coordinates: [
          [
            [139.8107, 35.7101],
            [139.7528, 35.6586],
          ],
          [
            [139.7, 35.68],
            [139.72, 35.69],
          ],
        ],
      },
      properties: {
        name: "Multi-Line Example",
      },
    },
    {
      type: "Feature",
      geometry: {
        type: "Polygon",
        coordinates: [
          [
            [139.75, 35.68],
            [139.76, 35.68],
            [139.76, 35.69],
            [139.75, 35.69],
            [139.75, 35.68],
          ],
        ],
      },
      properties: {
        name: "Single Zone",
      },
    },
    {
      type: "Feature",
      geometry: {
        type: "MultiPolygon",
        coordinates: [
          [
            [
              [139.77, 35.7],
              [139.78, 35.7],
              [139.78, 35.71],
              [139.77, 35.71],
              [139.77, 35.7],
            ],
          ],
          [
            [
              [139.79, 35.72],
              [139.8, 35.72],
              [139.8, 35.73],
              [139.79, 35.73],
              [139.79, 35.72],
            ],
          ],
        ],
      },
      properties: {
        name: "Multi-Zone",
      },
    },
  ],
};
