import * as Y from "yjs";

// JS to YJS
export const toYjsText = (text?: string | null): Y.Text | undefined =>
  text ? new Y.Text(text) : undefined;

export const toYjsArray = <T>(array?: unknown[]): Y.Array<T> | undefined => {
  if (!array) return;
  const yArray = new Y.Array();
  let hasValue = false;
  for (const value of array) {
    if (value === null || value === undefined) continue;
    yArray.push([value]);
    hasValue = true;
  }
  if (!hasValue) return;
  return yArray as Y.Array<T>;
};

export const toYjsMap = <T>(
  obj?: Record<string, unknown>,
): Y.Map<T> | undefined => {
  if (!obj) return;
  const yMap = new Y.Map();
  let hasValue = false;
  for (const [key, value] of Object.entries(obj)) {
    if (value === null || value === undefined) continue;
    yMap.set(key, value);
    hasValue = true;
  }
  if (!hasValue) return;
  return yMap as Y.Map<T>;
};

// YJS to JS
export const fromYjsText = (yText: Y.Text) => yText.toString();
export const fromYjsArray = (yArray: Y.Array<unknown>) => yArray.toArray();
export const fromYjsMap = (yMap: Y.Map<unknown>) => yMap.toJSON();
