/**
 * The engine's id for the intermediate data one output port left:
 * `[workflowPath.]nodeId.port`. It names the port's file in the feature store,
 * and the API takes it to identify the port.
 */
export const intermediateDataFileId = (
  nodeId: string,
  portName: string,
  workflowPath?: string,
): string => `${workflowPath ? `${workflowPath}.` : ""}${nodeId}.${portName}`;

const FEATURE_STORE_SEGMENT = "/feature-store/";
// Longest first, so the compressed form is matched before the plain one it
// ends with.
const FEATURE_FILE_EXTENSIONS = [".jsonl.zst", ".jsonl"];

/**
 * The file id read back out of a port's data URL, for selections stored before
 * the id was stored alongside the URL. Undefined when the URL is not a feature
 * store file.
 */
export const fileIdFromIntermediateDataUrl = (
  url: string,
): string | undefined => {
  const at = url.lastIndexOf(FEATURE_STORE_SEGMENT);
  if (at < 0) return undefined;

  let name = url.slice(at + FEATURE_STORE_SEGMENT.length).split(/[?#]/)[0];
  const extension = FEATURE_FILE_EXTENSIONS.find((ext) => name.endsWith(ext));
  if (!extension) return undefined;
  name = name.slice(0, -extension.length);

  try {
    name = decodeURIComponent(name);
  } catch {
    // Not percent-encoded after all; the name is usable as it stands.
  }
  return name || undefined;
};
