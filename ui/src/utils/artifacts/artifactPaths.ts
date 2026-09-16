/**
 * A job's outputs arrive as a flat list of absolute URLs — the server's
 * `ListJobArtifacts` walks the whole `artifacts/` prefix — but each URL still
 * carries the full object path. So the folder a writer with `groupBy` set
 * creates (one file per group, e.g. `out.geojson/<hash>.geojson`) is recovered
 * by cutting the URL at the job's artifact root, not lost.
 */

export type ArtifactFile = {
  /** Absolute URL the file is served from. */
  url: string;
  /** Path relative to the job's artifact root, e.g. `out.geojson/a3f9….geojson`. */
  path: string;
  /** Directory portion of `path`. Empty for files sitting at the artifact root. */
  folder: string;
  /** Final segment of `path`. */
  name: string;
};

type ArtifactFolder = {
  /** Directory these files sit in, relative to the artifact root. Empty for the root. */
  path: string;
  files: ArtifactFile[];
};

const ARTIFACT_ROOT = "/artifacts/";

/** Everything after the job's artifact root in `url`, percent-decoded. */
const relativePath = (url: string): string => {
  let pathname = url;
  try {
    pathname = new URL(url).pathname;
  } catch (_e) {
    // Not an absolute URL — read the whole string as a path.
  }

  // The layout is `artifacts/<jobId>/artifacts/<relative path>`, so the last
  // occurrence is the job's root — unless an output folder is itself named
  // `artifacts`, which would cost one level of nesting.
  const cut = pathname.includes(ARTIFACT_ROOT)
    ? pathname.slice(pathname.lastIndexOf(ARTIFACT_ROOT) + ARTIFACT_ROOT.length)
    : // No artifact root to cut at: only the file name can be trusted.
      (pathname.split("/").pop() ?? pathname);

  try {
    return decodeURIComponent(cut);
  } catch (_e) {
    // A stray `%` is not worth dropping the whole path over.
    return cut;
  }
};

export const toArtifactFiles = (urls: string[]): ArtifactFile[] =>
  urls.map((url) => {
    const segments = relativePath(url).split("/").filter(Boolean);
    return {
      url,
      path: segments.join("/"),
      folder: segments.slice(0, -1).join("/"),
      name: segments[segments.length - 1] ?? url,
    };
  });

/** Files bucketed by their folder, root first and each bucket sorted by name. */
export const groupArtifactsByFolder = (
  files: ArtifactFile[],
): ArtifactFolder[] => {
  const byFolder = new Map<string, ArtifactFile[]>();
  for (const file of files) {
    const existing = byFolder.get(file.folder);
    if (existing) {
      existing.push(file);
    } else {
      byFolder.set(file.folder, [file]);
    }
  }

  return [...byFolder.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([path, folderFiles]) => ({
      path,
      files: [...folderFiles].sort((a, b) => a.name.localeCompare(b.name)),
    }));
};

/**
 * Archive name for one folder's worth of files. The folder path is flattened
 * because a `/` cannot appear in a file name.
 */
export const folderArchiveName = (base: string, folderPath: string): string =>
  `${base}_${folderPath.replace(/\//g, "_")}.zip`;
