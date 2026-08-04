export type XYPosition = { x: number; y: number };

// Coerce anything that is not a finite number to `fallback`. Unlike `?? 0`,
// this catches NaN and Infinity (NaN ?? 0 === NaN). A non-finite node position
// persisted in the Yjs doc — e.g. from screenToFlowPosition running before the
// canvas is measured — otherwise reaches ReactFlow, produces a NaN viewport
// transform, and triggers an infinite render loop (React #185) that makes the
// project impossible to open.
export const toFiniteNumber = (value: unknown, fallback = 0): number =>
  typeof value === "number" && Number.isFinite(value) ? value : fallback;

export const toFinitePosition = (
  position?: { x?: unknown; y?: unknown } | null,
): XYPosition => ({
  x: toFiniteNumber(position?.x),
  y: toFiniteNumber(position?.y),
});
