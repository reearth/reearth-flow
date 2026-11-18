import type { CmsAsset } from "@flow/types/cmsIntegration";

export type CmsAssetFormat =
  | "citygml"
  | "geojson"
  | "json"
  | "csv"
  | "czml"
  | "gpkg"
  | "shapefile"
  | "model"
  | "image"
  | "archive"
  | "other";

export const CMS_ASSET_FORMATS: Record<
  Exclude<CmsAssetFormat, "other">,
  { label: string; extensions: string[] }
> = {
  citygml: { label: "CityGML", extensions: ["gml"] },
  geojson: { label: "GeoJSON", extensions: ["geojson"] },
  json: { label: "JSON", extensions: ["json"] },
  csv: { label: "CSV", extensions: ["csv", "tsv"] },
  czml: { label: "CZML", extensions: ["czml"] },
  gpkg: { label: "GeoPackage", extensions: ["gpkg"] },
  shapefile: { label: "Shapefile", extensions: ["shp"] },
  model: { label: "3D", extensions: ["glb", "gltf", "obj", "mtl"] },
  image: { label: "Image", extensions: ["jpg", "jpeg", "png", "tif", "tiff"] },
  archive: { label: "ZIP", extensions: ["zip", "7z"] },
};

export const extensionOf = (filename: string) => {
  const dot = filename.lastIndexOf(".");
  return dot === -1 ? "" : filename.slice(dot + 1).toLowerCase();
};

export const cmsAssetFormat = (filename: string): CmsAssetFormat => {
  const ext = extensionOf(filename);
  const match = Object.entries(CMS_ASSET_FORMATS).find(([, f]) =>
    f.extensions.includes(ext),
  );
  return (match?.[0] as CmsAssetFormat | undefined) ?? "other";
};

export type CmsAssetSort =
  | "createdAtDesc"
  | "createdAtAsc"
  | "nameAsc"
  | "nameDesc"
  | "sizeAsc"
  | "sizeDesc";

const compare: Record<CmsAssetSort, (a: CmsAsset, b: CmsAsset) => number> = {
  createdAtDesc: (a, b) => b.createdAt.localeCompare(a.createdAt),
  createdAtAsc: (a, b) => a.createdAt.localeCompare(b.createdAt),
  nameAsc: (a, b) => a.filename.localeCompare(b.filename),
  nameDesc: (a, b) => b.filename.localeCompare(a.filename),
  sizeAsc: (a, b) => a.size - b.size,
  sizeDesc: (a, b) => b.size - a.size,
};

export const filterAndSortCmsAssets = (
  assets: CmsAsset[],
  {
    keyword,
    format,
    sort,
  }: { keyword?: string; format?: CmsAssetFormat | "all"; sort: CmsAssetSort },
) => {
  const term = keyword?.trim().toLowerCase();
  return assets
    .filter(
      (a) =>
        (!term || a.filename.toLowerCase().includes(term)) &&
        (!format || format === "all" || cmsAssetFormat(a.filename) === format),
    )
    .sort(compare[sort]);
};
