import { cleanup } from "@testing-library/react";
import * as Y from "yjs";

import {
  fromYjsArray,
  fromYjsMap,
  fromYjsText,
  toYjsArray,
  toYjsMap,
  toYjsText,
} from ".";

afterEach(() => {
  cleanup();
});

describe("Yjs conversions: ", () => {
  test("from JavaScript object to Yjs map", () => {
    const yDoc = new Y.Doc();
    const root = yDoc.getMap("root") as Y.Map<any>;
    const obj = { a: 1, b: 2 };
    root.set("map-test", toYjsMap(obj));
    root.set("map-test-undefined", toYjsMap(undefined));

    expect(root.get("map-test").toJSON()).toStrictEqual({ a: 1, b: 2 });
    expect(root.get("map-test-undefined")).toStrictEqual(undefined);
  });

  test("from JavaScript array to Yjs array", () => {
    const yDoc = new Y.Doc();
    const root = yDoc.getMap("root");
    const array = [1, 2, 3];
    root.set("array-test", toYjsArray(array));
    root.set("array-test-undefined", toYjsArray(undefined));
    expect(
      (root.get("array-test") as Y.Array<typeof array>)?.toArray(),
    ).toStrictEqual([1, 2, 3]);
    expect(
      root.get("array-test-undefined") as Y.Array<typeof array>,
    ).toStrictEqual(undefined);
  });

  test("from JavaScript text to Yjs text", () => {
    const yDoc = new Y.Doc();
    const root = yDoc.getMap("root");
    const text = "hello";
    root.set("text-test", toYjsText(text));
    root.set("text-test-undefined", toYjsText(""));
    expect(root.get("text-test")?.toString()).toBe("hello");
    expect(root.get("text-test-undefined")).toBe(undefined);
  });

  test("from Yjs map to JavaScript object", () => {
    const yDoc = new Y.Doc();
    const yMap = yDoc.getMap("map-test");
    yMap.set("a", 1);
    yMap.set("b", 2);
    const obj = fromYjsMap(yMap);
    expect(obj).toStrictEqual({ a: 1, b: 2 });
  });

  test("from Yjs array to JavaScript array", () => {
    const yDoc = new Y.Doc();
    const yArray = yDoc.getArray("array-test");
    yArray.insert(0, [1, 2, 3]);
    const array = fromYjsArray(yArray);
    expect(array).toStrictEqual([1, 2, 3]);
  });

  test("from Yjs text to JavaScript text", () => {
    const yDoc = new Y.Doc();
    const yText = yDoc.getText("text-test");
    yText.insert(0, "hello");
    const text = fromYjsText(yText);
    expect(text).toBe("hello");
  });
});
