export const DEFAULT_ENTRY_GRAPH_ID = "main";
export const DEFAULT_EDGE_PORT = "default";
export const DEFAULT_ROUTING_PORT = "default";
export const DEFAULT_NODE_SIZE = { width: 150, height: 25 };
export const ALLOWED_WORKFLOW_FILE_EXTENSIONS = ".json, .yaml, .yml";
export const ALLOWED_PROJECT_IMPORT_EXTENSIONS = ".zip";
export const ALLOWED_ASSET_IMPORT_EXTENSIONS =
  ".csv, .geojson, .gml, .json, .tsv, .py, .zip, .obj, .mtl, .jpg, .jpeg, .png, .tif, .tiff, .glb, .gltf, .gpkg";
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
];

export const NODE_DIALOG_NAVIGATION_KEYS = [
  "enter", // select
  "arrowup", // up
  "arrowdown", // down
];

export const CURSOR_COLORS = [
  "#5b61d4", // Indigo - muted professional
  "#7c3acd", // Purple - softer tone
  "#0ea5be", // Cyan - more subdued
  "#059669", // Emerald - darker variant
  "#d97706", // Amber - less bright
  "#dc2626", // Red - deeper tone
  "#64748b", // Slate - neutral
  "#65a30d", // Lime - toned down
  "#c2410c", // Orange - muted warm
  "#be185d", // Pink - deeper rose
  "#0f766e", // Teal - darker blue-green
  "#7c2d12", // Brown - earthy tone
  "#166534", // Green - forest shade
  "#991b1b", // Dark red - professional
  "#075985", // Sky blue - muted variant
  "#581c87", // Purple - deep violet
  "#365314", // Olive - natural tone
  "#92400e", // Burnt orange - warm earth
];
