/**
 * Entry point for turning a parsed intermediate-data JSONL line into the
 * feature the debug panel renders.
 *
 * Two geometry models exist, and the format is detected per feature rather
 * than chosen per file, so a panel switching between files needs no state.
 *
 * `new-geometry` became the engine's default in #2343, so every new run writes
 * the new format and the legacy branch has no live traffic. It is kept because
 * the flag is explicitly temporary — its own TODO says "remove after
 * migration" — and reverting it would otherwise leave the panel rendering
 * attributes with no geometry and no map. Delete this branch once the flag is
 * gone from the engine, not before.
 */
import { isNextFormat } from "@flow/lib/intermediateData";

import { transformLegacyFeature } from "./transformLegacyFeature";
import {
  transformNextFeature,
  type TransformedFeature,
} from "./transformNextFeature";

export type { TransformedFeature } from "./transformNextFeature";

export function intermediateDataTransform(parsedData: any): TransformedFeature {
  return isNextFormat(parsedData)
    ? transformNextFeature(parsedData)
    : transformLegacyFeature(parsedData);
}
