export const supportedVisualizations = ["2d-map", "3d-map"] as const;

export type SupportedVisualizations = (typeof supportedVisualizations)[number];
