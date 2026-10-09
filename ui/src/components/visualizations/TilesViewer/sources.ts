/**
 * What rendered views draw beneath their data, and the credit each is owed.
 * Credits are kept here as data rather than taken from the services, so no
 * markup from elsewhere is ever put on the page.
 */

/** One credit: `prefix`, then `label` linked to `url`, then `suffix`. */
export type Credit = {
  prefix?: string;
  label: string;
  url: string;
  suffix?: string;
};

const OPENSTREETMAP: Credit = {
  prefix: "© ",
  label: "OpenStreetMap",
  url: "https://www.openstreetmap.org/copyright",
  suffix: " contributors",
};

export type BasemapId = "paper" | "satellite";

export type Basemap = {
  id: BasemapId;
  /** An imagery tile template. */
  url: string;
  /** The deepest zoom the source has tiles for; deeper ones are stretched. */
  maxZoom: number;
  credits: Credit[];
};

/** A light, plain map, so the data stands out. */
export const PAPER: Basemap = {
  id: "paper",
  url: "https://papers.reearth.land/styles/protomaps-light/tile/{z}/{x}/{y}.webp",
  maxZoom: 20,
  credits: [
    {
      label: "Re:Earth Papers",
      url: "https://papers.reearth.land/attribution",
    },
    OPENSTREETMAP,
  ],
};

// Heights are above the ellipsoid, as the views' own heights are meant to be.
export const TERRAIN_URL =
  "https://terrain.reearth.land/cesium-mesh/ellipsoid/{z}/{x}/{y}.terrain";
export const TERRAIN_MAX_ZOOM = 14;

/** As the terrain service's own `layer.json` lists them. */
export const TERRAIN_CREDITS: Credit[] = [
  { label: "Re:Earth Terrain", url: "https://terrain.reearth.land/" },
  { label: "Mapterhorn", url: "https://mapterhorn.com/" },
  { label: "EGM2008 (NGA)", url: "https://earth-info.nga.mil/" },
  { label: "Protomaps", url: "https://protomaps.com/" },
  OPENSTREETMAP,
];
