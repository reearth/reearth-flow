export const getBreakClass = (text: string): string =>
  /\s/.test(text) ? "break-words" : "break-all";
