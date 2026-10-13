import { useEffect, useMemo, useState } from "react";

import { config } from "@flow/config";
import { initializeSentinel } from "@flow/services/sentinel";

import type { Basemap } from "./sources";
import { PAPER } from "./sources";

// The same ground imagery the client-side map viewer uses.
const PUBLIC_IMAGERY_URL =
  "https://services.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}";

/**
 * The basemaps a rendered view can draw beneath its data, the default first:
 * the light map, then satellite imagery from the configured tile server once
 * its request signing is ready, or public imagery where none is configured.
 */
export const useBasemaps = (): Basemap[] => {
  const { tileServerBaseUrl, tileServerToken } = config();
  const signed = !!(tileServerBaseUrl && tileServerToken);
  const [ready, setReady] = useState(!signed);

  useEffect(() => {
    if (!signed) return;
    let cancelled = false;
    initializeSentinel().finally(() => {
      if (!cancelled) setReady(true);
    });
    return () => {
      cancelled = true;
    };
  }, [signed]);

  const satelliteUrl = !signed
    ? PUBLIC_IMAGERY_URL
    : ready && tileServerBaseUrl
      ? `${tileServerBaseUrl.replace(/\/$/, "")}/imagery/{z}/{x}/{y}.webp`
      : undefined;

  // Kept the same between renders, since the map is rebuilt when it changes.
  return useMemo(
    () =>
      satelliteUrl
        ? [
            PAPER,
            // Credits are not yet known for the satellite imagery.
            { id: "satellite", url: satelliteUrl, maxZoom: 19, credits: [] },
          ]
        : [PAPER],
    [satelliteUrl],
  );
};
