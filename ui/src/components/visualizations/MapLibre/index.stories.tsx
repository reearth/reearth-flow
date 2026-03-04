import type { Meta, StoryObj } from "@storybook/react-vite";

import {
  lineStringGeoJsonData,
  mixtureGeoJsonData,
  pointGeoJsonData,
  polygonGeoJsonData,
} from "./geoJsonExampleData";

import { MapLibre } from ".";

const meta = {
  component: MapLibre,
  tags: ["autodocs"],
  argTypes: {},
} satisfies Meta<typeof MapLibre>;

export default meta;
type Story = StoryObj<typeof meta>;

export const WithPoint: Story = {
  args: {
    className: "h-[500px]",
    fileContent: pointGeoJsonData,
    fileType: "geojson",
    mapRef: { current: null },
    showSelectedFeatureOnly: false,
    onMapLoad: () => {},
    onSelectedFeature: () => {},
    onShowFeatureDetailsOverlay: () => {},
  },
};

export const WithLineString: Story = {
  args: {
    className: "h-[500px]",
    fileContent: lineStringGeoJsonData,
    fileType: "geojson",
    mapRef: { current: null },
    showSelectedFeatureOnly: false,
    onMapLoad: () => {},
    onSelectedFeature: () => {},
    onShowFeatureDetailsOverlay: () => {},
  },
};

export const WithPolygon: Story = {
  args: {
    className: "h-[500px]",
    fileContent: polygonGeoJsonData,
    fileType: "geojson",
    mapRef: { current: null },
    showSelectedFeatureOnly: false,
    onMapLoad: () => {},
    onSelectedFeature: () => {},
    onShowFeatureDetailsOverlay: () => {},
  },
};

export const WithMixture: Story = {
  args: {
    className: "h-[500px]",
    fileContent: mixtureGeoJsonData,
    fileType: "geojson",
    mapRef: { current: null },
    showSelectedFeatureOnly: false,
    onMapLoad: () => {},
    onSelectedFeature: () => {},
    onShowFeatureDetailsOverlay: () => {},
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
    mapRef: { current: null },
    showSelectedFeatureOnly: false,
    onMapLoad: () => {},
    onSelectedFeature: () => {},
    onShowFeatureDetailsOverlay: () => {},
  },
};
