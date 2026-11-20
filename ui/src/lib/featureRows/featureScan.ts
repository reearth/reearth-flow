const TAB = 9;
const NEWLINE = 10;
const CARRIAGE_RETURN = 13;
const SPACE = 32;
const QUOTE = 34;
const PLUS = 43;
const COMMA = 44;
const MINUS = 45;
const DOT = 46;
const ZERO = 48;
const NINE = 57;
const COLON = 58;
const UPPER_E = 69;
const OPEN_BRACKET = 91;
const BACKSLASH = 92;
const CLOSE_BRACKET = 93;
const LOWER_E = 101;
const OPEN_BRACE = 123;
const CLOSE_BRACE = 125;

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8");

const ID_KEY = encoder.encode('"id"');
const ATTRIBUTES_KEY = encoder.encode('"attributes"');
const TRUE = encoder.encode("true");
const FALSE = encoder.encode("false");
const NULL = encoder.encode("null");
const ESCAPES = new Set(encoder.encode('"\\/bfnrtu'));

export type Span = [number, number];

export type FeatureSpans = {
  id: Span | null;
  attributes: Span | null;
};

const isDigit = (byte: number) => byte >= ZERO && byte <= NINE;

const isHex = (byte: number) =>
  isDigit(byte) || (byte >= 65 && byte <= 70) || (byte >= 97 && byte <= 102);

function keyIs(
  bytes: Uint8Array,
  start: number,
  end: number,
  key: Uint8Array,
  escaped: boolean,
) {
  if (escaped) {
    const name = JSON.parse(decoder.decode(bytes.subarray(start, end)));
    return name === decoder.decode(key.subarray(1, key.length - 1));
  }
  if (end - start !== key.length) return false;
  for (let i = 0; i < key.length; i++) {
    if (bytes[start + i] !== key[i]) return false;
  }
  return true;
}

const OBJECT = 1;
const ARRAY = 2;

const VALUE = 0;
const AFTER_VALUE = 1;
const KEY = 2;
const KEY_OR_CLOSE = 3;
const VALUE_OR_CLOSE = 4;

let escapedString = false;

function stringEnd(bytes: Uint8Array, pos: number, end: number) {
  escapedString = false;
  for (let i = pos + 1; i < end; i++) {
    const byte = bytes[i];
    if (byte === QUOTE) return i + 1;
    if (byte < SPACE) return -1;
    if (byte === BACKSLASH) {
      const escape = bytes[++i];
      if (i >= end || !ESCAPES.has(escape)) return -1;
      escapedString = true;
      if (escape === 117) {
        if (i + 4 >= end) return -1;
        for (let h = 1; h <= 4; h++) {
          if (!isHex(bytes[i + h])) return -1;
        }
        i += 4;
      }
    }
  }
  return -1;
}

function digitsEnd(bytes: Uint8Array, pos: number, end: number) {
  while (pos < end && isDigit(bytes[pos])) pos++;
  return pos;
}

function numberEnd(bytes: Uint8Array, pos: number, end: number) {
  if (bytes[pos] === MINUS) pos++;
  if (pos < end && bytes[pos] === ZERO) {
    pos++;
  } else {
    const next = digitsEnd(bytes, pos, end);
    if (next === pos) return -1;
    pos = next;
  }
  if (pos < end && bytes[pos] === DOT) {
    const next = digitsEnd(bytes, pos + 1, end);
    if (next === pos + 1) return -1;
    pos = next;
  }
  if (pos < end && (bytes[pos] === LOWER_E || bytes[pos] === UPPER_E)) {
    pos++;
    if (pos < end && (bytes[pos] === PLUS || bytes[pos] === MINUS)) pos++;
    const next = digitsEnd(bytes, pos, end);
    if (next === pos) return -1;
    pos = next;
  }
  return pos;
}

function literalEnd(bytes: Uint8Array, pos: number, end: number) {
  for (const word of [TRUE, FALSE, NULL]) {
    if (bytes[pos] !== word[0] || pos + word.length > end) continue;
    let match = true;
    for (let i = 1; i < word.length; i++) {
      if (bytes[pos + i] !== word[i]) {
        match = false;
        break;
      }
    }
    if (match) return pos + word.length;
  }
  return -1;
}

export function scanFeature(
  bytes: Uint8Array,
  start: number,
  end: number,
): FeatureSpans | null {
  const spans: FeatureSpans = { id: null, attributes: null };
  const stack: number[] = [];
  let state = VALUE;
  let pos = start;
  let keyStart = 0;
  let keyEnd = 0;
  let keyEscaped = false;
  let valueStart = 0;
  let opened = false;

  while (true) {
    let byte = bytes[pos];
    while (
      pos < end &&
      (byte === SPACE ||
        byte === TAB ||
        byte === NEWLINE ||
        byte === CARRIAGE_RETURN)
    ) {
      byte = bytes[++pos];
    }

    if (stack.length === 0 && state === AFTER_VALUE) {
      return pos === end ? spans : null;
    }
    if (pos >= end) return null;

    let completed = false;
    switch (state) {
      case KEY_OR_CLOSE:
      case KEY:
        if (state === KEY_OR_CLOSE && byte === CLOSE_BRACE) {
          stack.pop();
          pos++;
          completed = true;
          break;
        }
        if (byte !== QUOTE) return null;
        {
          const next = stringEnd(bytes, pos, end);
          if (next < 0) return null;
          if (stack.length === 1) {
            keyStart = pos;
            keyEnd = next;
            keyEscaped = escapedString;
          }
          pos = next;
        }
        while (pos < end && bytes[pos] !== COLON) {
          const space = bytes[pos];
          if (
            space !== SPACE &&
            space !== TAB &&
            space !== NEWLINE &&
            space !== CARRIAGE_RETURN
          ) {
            return null;
          }
          pos++;
        }
        if (pos >= end) return null;
        pos++;
        state = VALUE;
        break;
      case VALUE_OR_CLOSE:
      case VALUE: {
        if (state === VALUE_OR_CLOSE && byte === CLOSE_BRACKET) {
          stack.pop();
          pos++;
          completed = true;
          break;
        }
        if (stack.length === 1) valueStart = pos;
        if (byte === OPEN_BRACE) {
          if (stack.length === 0 && opened) return null;
          opened = true;
          stack.push(OBJECT);
          pos++;
          state = KEY_OR_CLOSE;
          break;
        }
        if (stack.length === 0) return null;
        if (byte === OPEN_BRACKET) {
          stack.push(ARRAY);
          pos++;
          state = VALUE_OR_CLOSE;
          break;
        }
        const next =
          byte === QUOTE
            ? stringEnd(bytes, pos, end)
            : byte === MINUS || isDigit(byte)
              ? numberEnd(bytes, pos, end)
              : literalEnd(bytes, pos, end);
        if (next < 0) return null;
        pos = next;
        completed = true;
        break;
      }
      case AFTER_VALUE: {
        const container = stack[stack.length - 1];
        pos++;
        if (byte === COMMA) {
          state = container === OBJECT ? KEY : VALUE;
          break;
        }
        if (
          (byte === CLOSE_BRACE && container === OBJECT) ||
          (byte === CLOSE_BRACKET && container === ARRAY)
        ) {
          stack.pop();
          completed = true;
          break;
        }
        return null;
      }
    }

    if (!completed) continue;
    state = AFTER_VALUE;
    if (stack.length !== 1) continue;
    if (keyIs(bytes, keyStart, keyEnd, ID_KEY, keyEscaped)) {
      spans.id = [valueStart, pos];
    } else if (keyIs(bytes, keyStart, keyEnd, ATTRIBUTES_KEY, keyEscaped)) {
      spans.attributes = [valueStart, pos];
    }
    if (spans.id && spans.attributes) return spans;
  }
}
