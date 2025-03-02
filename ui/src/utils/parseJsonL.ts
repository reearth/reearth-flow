/**
 * Parses a JSONL (JSON Lines) string into an array of objects.
 *
 * @param text - The JSONL string where each line is a JSON object.
 * @param options - Optional configuration options.
 * @param options.transform - A function to transform each parsed object.
 *                           It receives the parsed object and its line index as arguments.
 * @param options.onError - Custom error handler for parse errors.
 *                         Receives the error, line content, and line index.
 * @param options.skipEmpty - Whether to skip empty lines (default: true).
 * @param options.trimLines - Whether to trim whitespace from lines (default: true).
 * @returns An array of parsed (and optionally transformed) objects.
 */
export const parseJSONL = <T = any>(
  text: string,
  options?: {
    transform?: (obj: any, index: number) => T;
    onError?: (error: Error, line: string, index: number) => void;
    skipEmpty?: boolean;
    trimLines?: boolean;
  },
): T[] => {
  const {
    transform,
    onError,
    skipEmpty = true,
    trimLines = true,
  } = options || {};

  // Handle empty input
  if (!text) return [];

  const lines = text.split(/\r?\n/); // Handle both CRLF and LF line endings
  const result: T[] = [];

  for (let i = 0; i < lines.length; i++) {
    let line = lines[i];

    if (trimLines) {
      line = line.trim();
    }

    if (skipEmpty && !line) {
      continue;
    }

    try {
      const parsed = JSON.parse(line);
      result.push(transform ? transform(parsed, i) : parsed);
    } catch (error) {
      if (onError) {
        onError(error as Error, line, i);
      } else {
        console.error(`Failed to parse line ${i}:`, line, error);
      }
    }
  }

  return result;
};
