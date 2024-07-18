import * as Y from "yjs";

export const toYjsMap = (map: Map<string, any>) => {
  const yMap = new Y.Map();
  map.forEach((value, key) => {
    yMap.set(key, value);
  });
  return yMap;
};

export const fromYjsMap = (yMap: Y.Map<any>) => {
  const map = new Map();
  yMap.forEach((value, key) => {
    map.set(key, value);
  });
  return map;
};

export const toYjsArray = (array: any[]) => {
  const yArray = new Y.Array();
  array.forEach((value, index) => {
    yArray.insert(index, [value]);
  });
  return yArray;
};

export const toYjsText = (text: string) => {
  const yText = new Y.Text();
  yText.insert(0, text);
  return yText;
};

export const toYjsXmlFragment = (xml: string) => {
  const yXmlFragment = new Y.XmlFragment();
  const yXmlText = new Y.XmlText(xml);
  yXmlFragment.insert(0, [yXmlText]);
  return yXmlFragment;
};

export const toYjsXmlText = (xml: string) => {
  const yXmlText = new Y.XmlText();
  yXmlText.insert(0, xml);
  return yXmlText;
};

export const toYjsXmlElement = (tagName: string) => {
  const yXmlElement = new Y.XmlElement(tagName);
  return yXmlElement;
};

