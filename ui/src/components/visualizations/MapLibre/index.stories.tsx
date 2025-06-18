import type { Meta, StoryObj } from "@storybook/react";

import { MapLibre } from ".";

const meta = {
  component: MapLibre,
  tags: ["autodocs"],
  argTypes: {},
} satisfies Meta<typeof MapLibre>;

export default meta;
type Story = StoryObj<typeof meta>;

const pointGeoJsonData = {
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

const polygonGeoJsonData = {
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

const lineStringGeoJsonData = {
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

const mixtureGeoJsonData = {
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

export const WithPoint: Story = {
  args: {
    className: "h-[500px]",
    fileContent: pointGeoJsonData,
    fileType: "geojson",
  },
};

export const WithLineString: Story = {
  args: {
    className: "h-[500px]",
    fileContent: lineStringGeoJsonData,
    fileType: "geojson",
  },
};

export const WithPolygon: Story = {
  args: {
    className: "h-[500px]",
    fileContent: polygonGeoJsonData,
    fileType: "geojson",
  },
};

export const WithMixture: Story = {
  args: {
    className: "h-[500px]",
    fileContent: mixtureGeoJsonData,
    fileType: "geojson",
  },
};

export const EmptyData: Story = {
  args: {
    className: "h-[500px]",
    fileContent: {
      type: "FeatureCollection",
      features: [],
    },
    fileType: "geojson",
  },
};
