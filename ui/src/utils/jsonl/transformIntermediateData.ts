/**
 * Entry point for turning a parsed intermediate-data JSONL line into the
 * feature the debug panel renders.
 *
 * Two geometry models are in circulation. The engine's `new-geometry` feature
 * is off by default, and files written before it was enabled stay in the
 * artifact bucket regardless, so the format is detected per feature rather
 * than chosen per file — a run's output is uniform, but the panel switches
 * between files freely.
 */
import { isNextFormat } from "@flow/lib/intermediateData";

import { transformLegacyFeature } from "./transformLegacyFeature";
import {
  transformNextFeature,
  type NextTransformOptions,
  type TransformedFeature,
} from "./transformNextFeature";

export type { TransformedFeature } from "./transformNextFeature";

export function intermediateDataTransform(
  parsedData: any,
  options?: Partial<NextTransformOptions>,
): TransformedFeature {
  if (isNextFormat(parsedData)) {
    return transformNextFeature(parsedData, {
      owner: options?.owner ?? "",
      rowIndex: options?.rowIndex,
    });
  }

  const transformed = transformLegacyFeature(parsedData);
  return options?.rowIndex === undefined
    ? transformed
    : { ...transformed, rowIndex: options.rowIndex };
}
