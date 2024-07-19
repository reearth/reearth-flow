import * as Y from "yjs";

import { toYjsArray, toYjsMap, toYjsText } from "./conversions";

const doc = new Y.Doc();

test("convert from JavaScript object to Yjs map", () => {
  const obj = { a: 1, b: 2 };
  const yMap = toYjsMap(obj);
  doc.getMap("root").set("map-test", yMap);
  expect(yMap.toJSON()).toStrictEqual({ a: 1, b: 2 });
});

test("convert from Yjs map to JavaScript object", () => {
  const yMap = doc.getMap("map-test");
  yMap.set("a", 1);
  yMap.set("b", 2);
  const obj = yMap.toJSON();
  expect(obj).toStrictEqual({ a: 1, b: 2 });
});

test("convert from JavaScript array to Yjs array", () => {
  const array = [1, 2, 3];
  const yArray = toYjsArray(array);
  doc.getMap("root").set("array-test", yArray);
  expect(yArray.toJSON()).toStrictEqual([1, 2, 3]);
});

test("convert from Yjs array to JavaScript array", () => {
  const yArray = doc.getArray("array-test");
  yArray.insert(0, [1, 2, 3]);
  const array = yArray.toJSON();
  expect(array).toStrictEqual([1, 2, 3]);
});

test("convert from JavaScript text to Yjs text", () => {
  const text = "hello";
  const yText = toYjsText(text);
  doc.getMap("root").set("text-test", yText);
  expect(yText.toString()).toBe("hello");
});

test("convert from Yjs text to JavaScript text", () => {
  const yText = doc.getText("text-test");
  yText.insert(0, "hello");
  const text = yText.toString();
  expect(text).toBe("hello");
});

test("convert from JavaScript string to Yjs xml text", () => {
  const xml = "<hello />";
  const yXmlText = toYjsText(xml);
  doc.getMap("root").set("xml-text-test", yXmlText);
  expect(yXmlText.toString()).toBe("<hello />");
});

test("convert from Yjs xml text to JavaScript string", () => {
  const yXmlText = doc.getText("xml-text-test");
  yXmlText.insert(0, "<hello />");
  const xml = yXmlText.toString();
  expect(xml).toBe("<hello />");
});

test("convert from JavaScript string to Yjs xml fragment", () => {
  const xml = "<hello />";
  const yXmlFragment = toYjsText(xml);
  doc.getMap("root").set("xml-fragment-test", yXmlFragment);
  expect(yXmlFragment.toString()).toBe("<hello />");
});

test("convert from Yjs xml fragment to JavaScript string", () => {
  const yXmlFragment = doc.getText("xml-fragment-test");
  yXmlFragment.insert(0, "<hello />");
  const xml = yXmlFragment.toString();
  expect(xml).toBe("<hello />");
});
