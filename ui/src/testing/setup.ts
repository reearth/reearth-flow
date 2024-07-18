import { JSDOM } from 'jsdom';

const { window } = new JSDOM('<!doctype html><html><body></body></html>');

// Assign the window object to global
global.window = window as any;
global.document = window.document;
global.navigator = {
  userAgent: 'node.js',
} as any;

// Optional: if you need other global properties, you can assign them here
global.HTMLElement = window.HTMLElement;
global.Node = window.Node;
global.NodeList = window.NodeList;