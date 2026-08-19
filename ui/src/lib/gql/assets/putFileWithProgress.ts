export type UploadProgress = {
  loaded: number;
  total: number;
  /** 0-100. Undefined while the total transfer size is not yet known. */
  percent?: number;
};

export type UploadResult = {
  status: number;
  statusText: string;
  responseText: string;
};

type PutFileArgs = {
  url: string;
  file: File;
  headers?: Record<string, string | undefined | null>;
  stallTimeoutMs?: number;
  onProgress?: (progress: UploadProgress) => void;
};

/**
 * PUTs a file to a URL, reporting upload progress as it goes.
 *
 * Uses XMLHttpRequest rather than fetch because fetch cannot report progress on
 * a request body — it only resolves once the whole response is available, which
 * leaves a multi-megabyte upload indistinguishable from a hung one.
 */
export function putFileWithProgress({
  url,
  file,
  headers,
  stallTimeoutMs,
  onProgress,
}: PutFileArgs): Promise<UploadResult> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    // Guards against settling twice, since aborting on stall also fires onabort.
    let settled = false;
    let stallTimer: ReturnType<typeof setTimeout> | undefined;

    const clearStallTimer = () => {
      if (stallTimer) clearTimeout(stallTimer);
      stallTimer = undefined;
    };

    const fail = (message: string) => {
      if (settled) return;
      settled = true;
      clearStallTimer();
      reject(new Error(message));
    };

    const succeed = () => {
      if (settled) return;
      settled = true;
      clearStallTimer();
      resolve({
        status: xhr.status,
        statusText: xhr.statusText,
        responseText: xhr.responseText,
      });
    };

    const armStallTimer = () => {
      if (!stallTimeoutMs) return;
      clearStallTimer();
      stallTimer = setTimeout(() => {
        const minutes = Math.round(stallTimeoutMs / 60_000);
        // Reject before aborting: abort() synchronously fires onabort, which
        // would otherwise settle the promise with the generic abort message and
        // lose the real reason.
        fail(
          `transfer stalled — no bytes sent for ${minutes} minute${minutes === 1 ? "" : "s"}`,
        );
        xhr.abort();
      }, stallTimeoutMs);
    };

    xhr.open("PUT", url, true);

    Object.entries(headers ?? {}).forEach(([name, value]) => {
      // Skip empty values: an empty string is still sent as a real header.
      if (value) xhr.setRequestHeader(name, value);
    });

    xhr.upload.onprogress = (event) => {
      armStallTimer();
      const total = event.lengthComputable ? event.total : file.size;
      onProgress?.({
        loaded: event.loaded,
        total,
        percent: total
          ? Math.min(100, Math.round((event.loaded / total) * 100))
          : undefined,
      });
    };

    // The body is sent; anything after this is the server's turn, so a stall
    // here is a slow response rather than a stuck upload.
    xhr.upload.onload = () => clearStallTimer();

    xhr.onload = () => succeed();
    xhr.onerror = () =>
      fail(
        "the request never reached storage (connectivity, CORS, or a blocked request)",
      );
    xhr.ontimeout = () => fail("the request timed out");
    xhr.onabort = () => fail("the upload was aborted");

    armStallTimer();
    xhr.send(file);
  });
}
