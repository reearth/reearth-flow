export const DEFAULT_ENTRY_GRAPH_ID = "main";
export const DEFAULT_EDGE_PORT = "default";
export const DEFAULT_ROUTING_PORT = "default";
export const DEFAULT_NODE_SIZE = { width: 150, height: 25 };
export const ALLOWED_WORKFLOW_FILE_EXTENSIONS = ".json, .yaml, .yml";
export const ALLOWED_PROJECT_IMPORT_EXTENSIONS = ".zip";
export const ALLOWED_ASSET_IMPORT_EXTENSIONS =
  ".csv, .czml, .geojson, .glb, .gltf, .gml, .gpkg, .jpg, .jpeg, .json, .mtl, .obj, .png, .py, .tif, .tiff, .tsv, .zip";
export const CLIPBOARD_EXPIRATION_TIME = 1000 * 60 * 5;
export const GENERAL_HOT_KEYS = ["ctrl+slash", "meta+slash"];
export const GLOBAL_HOT_KEYS = [
  "equal", // zoom in
  "minus", // zoom out
  "meta+0", // fit view mac
  "ctrl+0", // fit view win
  "meta+f", // fullscreen mac
  "ctrl+f", // fullscreen win
];

export const CANVAS_HOT_KEYS = [
  "r", // reader dialog
  "t", // transformer dialog
  "w", //  writer dialog
  "meta+c", // copy mac
  "ctrl+c", // copy win
  "meta+x", // cut mac
  "ctrl+x", // cut win
  "meta+v", // paste mac
  "ctrl+v", // paste win
];
export const EDITOR_HOT_KEYS = [
  "shift+meta+z", // redo mac
  "shift+ctrl+z", // redo win
  "meta+z", // undo mac
  "ctrl+z", // undo win
  "meta+s", // save mac
  "ctrl+s", // save win
  "shift+meta+s", // add subworkflow from selection mac
  "shift+ctrl+s", // add subworkflow from selection win
];

export const CURSOR_COLORS = [
  "#5b61d460", // Indigo - muted professional
  "#7c3acd60", // Purple - softer tone
  "#0ea5be60", // Cyan - more subdued
  "#05966960", // Emerald - darker variant
  "#d9770660", // Amber - less bright
  "#dc262660", // Red - deeper tone
  "#64748b60", // Slate - neutral
  "#65a30d60", // Lime - toned down
  "#c2410c60", // Orange - muted warm
  "#be185d60", // Pink - deeper rose
  "#0f766e60", // Teal - darker blue-green
  "#7c2d1260", // Brown - earthy tone
  "#16653460", // Green - forest shade
  "#991b1b60", // Dark red - professional
  "#07598560", // Sky blue - muted variant
  "#581c8760", // Purple - deep violet
  "#36531460", // Olive - natural tone
  "#92400e60", // Burnt orange - warm earth
];
