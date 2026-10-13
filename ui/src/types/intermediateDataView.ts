/**
 * What was asked for: one row as a 3D model, or the whole port as tiles. The
 * user picks this; the engine picks the format.
 */
export type IntermediateDataViewShape = "gltf" | "tiles";

/**
 * What the engine produced. A tiles view is 3D Tiles if any of its geometry is
 * 3D and vector tiles only if all of it is 2D, so the format of a tiles view is
 * not known until it has rendered.
 */
export type IntermediateDataViewFormat =
  "glb" | "cesium3dTiles" | "vectorTiles";

/** Every status is final: a render is awaited in the request. */
export type IntermediateDataViewStatus =
  "ready" | "empty" | "unsupportedGeometry" | "failed";

export type IntermediateDataView = {
  /** A hash of everything that affects the output, so the same request always yields the same id. */
  id: string;
  jobId: string;
  /** The engine's `[subgraphPrefix.]nodeId.port`. */
  fileId: string;
  shape: IntermediateDataViewShape;
  status: IntermediateDataViewStatus;
  /** Only once a render has finished. */
  format?: IntermediateDataViewFormat;
  /** Only when the status is ready. */
  entryPointUrl?: string;
  /** Features the selection kept. Missing when the render's report could not be read. */
  selectedFeatures?: number;
  /** Features drawn into the view, which can be fewer than were selected. */
  renderedFeatures?: number;
  /** Vector tiles only: features left out of at least one tile to keep it under the size limit. */
  sizeLimitedFeatures?: number;
  /** Vector tiles only: tiles that left out at least one feature to stay under the size limit. */
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
  | "unknown";
