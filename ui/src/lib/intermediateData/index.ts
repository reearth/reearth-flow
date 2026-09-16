/**
 * Reading the engine's intermediate-data format.
 *
 * Only what the rest of the app uses is re-exported here; the schema maps the
 * labels are derived from are internal to this directory.
 */
export {
  describeGeometry,
  isNextFormat,
  type GeometryDescription,
  type GeometryKind,
} from "./labels";
