/**
 * Reading the engine's intermediate-data format.
 *
 * Only what the rest of the app uses is re-exported here; the schema maps and
 * the label/raster helpers the walk depends on are internal to this directory
 * and imported directly by the modules that need them.
 */
export {
  describeGeometry,
  isNextFormat,
  type GeometryDescription,
  type GeometryKind,
} from "./labels";
export {
  extractAppearance,
  type AppearanceSummary,
  type MaterialSummary,
  type TextureSlot,
} from "./rasters";
export {
  acquireObjectUrl,
  getRasterInfo,
  isRasterHandle,
  RASTER_REF,
  releaseObjectUrl,
  releaseOwner,
  type RasterHandle,
  type RasterInfo,
} from "./rasterStore";
/** Test-only: clearing the store between cases, and asserting what it holds. */
export { clearRasterStore, retainedRasterBytes } from "./rasterStore";
