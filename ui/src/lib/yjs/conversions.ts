import * as Y from "yjs";

export const toYjsMap = (obj: Record<string, unknown>): Y.Map<unknown> => {
  const yMap = new Y.Map();
  for (const [key, value] of Object.entries(obj)) {
    yMap.set(key, value);
  }
  return yMap;
};

export const fromYjsMap = (yMap: Y.Map<unknown>) => {
  const map = new Map();
  yMap.forEach((value, key) => {
    map.set(key, value);
  });
  return map;
};

export const toYjsArray = (array: unknown[]) => {
  const yArray = new Y.Array();
  array.forEach((value, index) => {
    yArray.insert(index, [value]);
  });
  return yArray;
};

export const fromYjsArray = (yArray: Y.Array<unknown>) => {
  return yArray.toArray();
};

export const toYjsText = (text: string) => {
  const yText = new Y.Text();
  yText.insert(0, text);
  return yText;
};

export const fromYjsText = (yText: Y.Text) => {
  return yText.toString();
};

export const toYjsXmlFragment = (xml: string) => {
  const yXmlFragment = new Y.XmlFragment();
  const yXmlText = new Y.XmlText(xml);
  yXmlFragment.insert(0, [yXmlText]);
  return yXmlFragment;
};

export const fromYjsXmlFragment = (yXmlFragment: Y.XmlFragment) => {
  return yXmlFragment.toString();
};

export const toYjsXmlText = (xml: string) => {
  const yXmlText = new Y.XmlText();
  yXmlText.insert(0, xml);
  return yXmlText;
};

export const fromYjsXmlText = (yXmlText: Y.XmlText) => {
  return yXmlText.toString();
};

export const toYjsXmlElement = (tagName: string) => {
  const yXmlElement = new Y.XmlElement(tagName);
  return yXmlElement;
};

export const fromYjsXmlElement = (yXmlElement: Y.XmlElement) => {
  return yXmlElement.toString();
};
