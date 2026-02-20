import { Color } from "cesium";
import { useCallback, useEffect, useRef } from "react";

import type { CityGmlTypeConfig } from "./cityGmlGeometryToPrimitives";
import type { PolygonInput, WorkerOutput } from "./lodGeometryWorker";

// ── Types ────────────────────────────────────────────────────────────────────

type CityGmlFeature = {
  id?: string;
  type: "Feature";
  properties: Record<string, any>;
  geometry: { type: "CityGmlGeometry"; [key: string]: any };
};

export type LodGeometryResult = WorkerOutput;

// ── Surface type color constants (cached, never recreated) ───────────────────

const FLOOR_COLOR: [number, number, number, number] = [
  Color.BROWN.red,
  Color.BROWN.green,
  Color.BROWN.blue,
  0.8,
];
const ROOF_COLOR: [number, number, number, number] = [
  Color.RED.red,
  Color.RED.green,
  Color.RED.blue,
  0.8,
];
const WALL_COLOR: [number, number, number, number] = [
  Color.BLUE.red,
  Color.BLUE.green,
  Color.BLUE.blue,
  0.8,
];
const GRAY_COLOR: [number, number, number, number] = [
  Color.GRAY.red,
  Color.GRAY.green,
  Color.GRAY.blue,
  0.8,
];
// ── Color helpers (pure data, no Cesium object creation) ─────────────────────

function getSurfaceTypeColorTuple(
  exterior: any[],
  globalMinZ: number,
): [number, number, number, number] {
  let minZ = Infinity;
  let maxZ = -Infinity;
  for (const coord of exterior) {
    const z = coord.z || 0;
    if (z < minZ) minZ = z;
    if (z > maxZ) maxZ = z;
  }
  const isFlat = Math.abs(maxZ - minZ) < 0.1;
  if (isFlat && minZ < globalMinZ + 1) return FLOOR_COLOR;
  if (isFlat) return ROOF_COLOR;
  return WALL_COLOR;
}

function resolveAppearanceColorTuple(
  globalIndex: number,
  geometry: any,
  defaultColor: [number, number, number, number],
): [number, number, number, number] {
  const polygonMaterials = geometry.polygonMaterials;
  const materials = geometry.materials;
  if (
    globalIndex >= 0 &&
    Array.isArray(polygonMaterials) &&
    Array.isArray(materials) &&
    materials.length > 0
  ) {
    const matIdx = polygonMaterials[globalIndex];
    if (matIdx != null && materials[matIdx]) {
      const mat = materials[matIdx];
      const [r, g, b] = (mat.diffuseColor as number[]) ?? [1, 1, 1];
      const alpha = 1 - ((mat.transparency as number) ?? 0);
      return [r, g, b, alpha];
    }
  }
  return defaultColor;
}

// ── Prepare worker input from a feature ──────────────────────────────────────

function prepareWorkerInput(
  feature: CityGmlFeature,
  typeConfig: CityGmlTypeConfig | undefined,
  requestId: number,
): { input: import("./lodGeometryWorker").WorkerInput } | null {
  const { geometry } = feature;
  const gmlGeometries =
    geometry.gmlGeometries || geometry.value?.cityGmlGeometry?.gmlGeometries;
  if (!gmlGeometries || !Array.isArray(gmlGeometries)) return null;

  // LOD3 first, fallback to LOD2
  let lodGeometries = gmlGeometries.filter(
    (geom: any) =>
      geom.lod === 3 ||
      geom.gml_trait?.property?.includes("Lod3") ||
      geom.gml_trait?.property?.includes("LOD3"),
  );
  if (lodGeometries.length === 0) {
    lodGeometries = gmlGeometries.filter(
      (geom: any) =>
        geom.lod === 2 ||
        geom.gml_trait?.property?.includes("Lod2") ||
        geom.gml_trait?.property?.includes("LOD2"),
    );
  }
  if (lodGeometries.length === 0) return null;

  // Collect polygons with global indices
  const allPolygons: { polygon: any; globalIndex: number }[] = [];
  for (const geom of lodGeometries) {
    if (geom.polygons && Array.isArray(geom.polygons)) {
      const baseIndex: number = geom.pos ?? 0;
      for (let i = 0; i < geom.polygons.length; i++) {
        allPolygons.push({
          polygon: geom.polygons[i],
          globalIndex: baseIndex + i,
        });
      }
    }
  }
  if (allPolygons.length === 0) return null;

  // Compute globalMinZ
  let globalMinZ = Infinity;
  for (const { polygon } of allPolygons) {
    for (const c of polygon.exterior || []) {
      const z = c.z || 0;
      if (z < globalMinZ) globalMinZ = z;
    }
  }
  if (globalMinZ === Infinity) return null;

  // Build color tuple for type config
  const typeColorTuple: [number, number, number, number] = typeConfig
    ? [
        typeConfig.color.red,
        typeConfig.color.green,
        typeConfig.color.blue,
        typeConfig.color.alpha,
      ]
    : GRAY_COLOR;

  // Convert to worker polygon format
  const workerPolygons: PolygonInput[] = [];
  for (const { polygon, globalIndex } of allPolygons) {
    if (!polygon.exterior || !Array.isArray(polygon.exterior)) continue;

    const defaultColor = typeConfig?.useSurfaceTypeColors
      ? getSurfaceTypeColorTuple(polygon.exterior, globalMinZ)
      : typeColorTuple;

    const color =
      typeConfig?.displayName === "Building"
        ? resolveAppearanceColorTuple(globalIndex, geometry, defaultColor)
        : defaultColor;

    workerPolygons.push({
      exterior: polygon.exterior,
      globalIndex,
      color,
    });
  }

  if (workerPolygons.length === 0) return null;

  return {
    input: { requestId, polygons: workerPolygons, globalMinZ },
  };
}

// ── Hook ─────────────────────────────────────────────────────────────────────

export function useLodWorker() {
  const workerRef = useRef<Worker | null>(null);
  const requestIdRef = useRef(0);
  const pendingRef = useRef<Map<number, (result: WorkerOutput) => void>>(
    new Map(),
  );

  // Create worker on mount, terminate on unmount
  useEffect(() => {
    const worker = new Worker(
      new URL("./lodGeometryWorker.ts", import.meta.url),
      { type: "module" },
    );
    const pending = pendingRef.current;

    worker.onmessage = (e: MessageEvent<WorkerOutput>) => {
      const { requestId } = e.data;
      const resolve = pending.get(requestId);
      if (resolve) {
        pending.delete(requestId);
        resolve(e.data);
      }
      // If no resolver found, this was a stale/cancelled request — silently drop
    };

    workerRef.current = worker;

    return () => {
      worker.terminate();
      workerRef.current = null;
      pending.clear();
    };
  }, []);

  const buildLodGeometry = useCallback(
    (
      feature: CityGmlFeature,
      typeConfig: CityGmlTypeConfig | undefined,
    ): Promise<WorkerOutput> | null => {
      const worker = workerRef.current;
      if (!worker) return null;

      const requestId = ++requestIdRef.current;
      const prepared = prepareWorkerInput(feature, typeConfig, requestId);
      if (!prepared) return null;

      return new Promise<WorkerOutput>((resolve) => {
        pendingRef.current.set(requestId, resolve);
        worker.postMessage(prepared.input);
      });
    },
    [],
  );

  const cancelPending = useCallback(() => {
    // Increment request ID so any in-flight responses are ignored
    requestIdRef.current++;
    pendingRef.current.clear();
  }, []);

  return { buildLodGeometry, cancelPending };
}
