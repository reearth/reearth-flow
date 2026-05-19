import { IconName } from "@flow/components";

export const getIconFileType = (
  ext: string | undefined,
): IconName | undefined => {
  switch (ext) {
    case "csv":
      return "fileCSV";
    case "czml":
      return "fileCzml";
    case "geojson":
      return "fileGeoJSON";
    case "glb":
      return "fileGlb";
    case "gltf":
      return "fileGltf";
    case "gml":
      return "fileGml";
    case "gpkg":
      return "fileGpkg";
    case "jpg":
      return "fileJpg";
    case "jpeg":
      return "fileJpeg";
    case "json":
      return "fileJson";
    case "mtl":
      return "fileMtl";
    case "obj":
      return "fileObj";
    case "png":
      return "filePng";
    case "py":
      return "filePy";
    case "tif":
      return "fileTif";
    case "tiff":
      return "fileTiff";
    case "tsv":
      return "fileTsv";
    case "zip":
      return "fileZip";
    default:
      return undefined;
  }
};
