/**
 * What was asked for: one row as a 3D model, or the whole port as tiles. The
 * user picks this; the engine picks the format.
 */
export type IntermediateDataViewShape = "gltf" | "tiles";

/**
 * What the engine produced. A tiles view draws its 3D geometry as 3D Tiles and
 * its 2D geometry as vector tiles, so the formats of a tiles view are not known
 * until it has rendered.
 */
export type IntermediateDataViewFormat =
  "glb" | "cesium3dTiles" | "vectorTiles";

/** Every status is final: a render is awaited in the request. */
export type IntermediateDataViewStatus =
  "ready" | "empty" | "unsupportedGeometry" | "failed";

/** One entry point of a ready view, and what was drawn into it. */
export type IntermediateDataViewLayer = {
  format: IntermediateDataViewFormat;
  entryPointUrl: string;
  /**
   * Features this layer was given: the whole selection, unless the view has two
   * layers, when it is the features with parts of this layer's dimension.
   */
  selectedFeatures: number;
  /** Of those, the features drawn into this layer. */
  renderedFeatures: number;
  /** Vector tiles only: features left out of at least one tile to keep it under the size limit. */
  sizeLimitedFeatures?: number;
  /** Vector tiles only: tiles that left out at least one feature to stay under the size limit. */
  sizeLimitedTiles?: number;
};

export type IntermediateDataView = {
  /** A hash of everything that affects the output, so the same request always yields the same id. */
  id: string;
  jobId: string;
  /** The engine's `[subgraphPrefix.]nodeId.port`. */
  fileId: string;
  shape: IntermediateDataViewShape;
  status: IntermediateDataViewStatus;
  /**
   * What a viewer opens, one per format, 3D Tiles first: a tiles view of
   * geometry holding both 3D and 2D has both. Empty unless the status is ready.
   */
  layers: IntermediateDataViewLayer[];
  /** Features the selection kept. Missing when the render's report could not be read. */
  selectedFeatures?: number;
  /**
   * Features drawn into the view, in any of its layers, which can be fewer than
   * were selected. A feature drawn into two layers counts once.
   */
  renderedFeatures?: number;
  /** The vector tile layer's sizeLimitedFeatures, when the view has one. */
  sizeLimitedFeatures?: number;
  /** The vector tile layer's sizeLimitedTiles, when the view has one. */
  sizeLimitedTiles?: number;
  /** Why there is no view, for any status but ready. */
  error?: string;
};

/**
 * The view of one port's intermediate data: the whole port as tiles, or a
 * single row as a 3D model when `row` is set.
 */
export type IntermediateDataViewRequest = {
  jobId: string;
  fileId: string;
  /** The row's 0-based line in the port's file, not its position in a sorted or searched table. */
  row?: number;
};

/** Why a view could not be requested, as opposed to a view that rendered and reports a non-ready status. */
export type IntermediateDataViewErrorKind =
  /** The renderer never answered in time. */
  | "timedOut"
  /** This deployment cannot render views. */
  | "unavailable"
  /** The run is still going, so its data may be incomplete. */
  | "jobNotFinished"
  /** The port left no intermediate data. */
  | "dataNotFound"
  /** The user may not render views in this workspace. */
  | "permissionDenied"
  /** This tab already has as many renders running as it allows. */
  | "tooManyRenders"
  /** Every server that renders views was busy, so nothing ran. Asking again shortly may work. */
  | "workersBusy"
  | "unknown";
