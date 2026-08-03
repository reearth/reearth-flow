export {
  canContainRaster,
  definitionLabel,
  describeGeometry,
  isNextFormat,
  propertyLabel,
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
  clearRasterStore,
  getRasterInfo,
  isRasterHandle,
  RASTER_REF,
  releaseObjectUrl,
  releaseOwner,
  retainedRasterBytes,
  type RasterHandle,
  type RasterInfo,
} from "./rasterStore";
