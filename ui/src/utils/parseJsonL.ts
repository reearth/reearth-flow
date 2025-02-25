/**
 * Parses a JSONL (JSON Lines) string into an array of objects.
 *
 * @param text - The JSONL string where each line is a JSON object.
 * @param transform - Optional. A function to transform each parsed object.
 *                    It receives the parsed object and its line index as arguments.
 *                    If not provided, the raw parsed object is returned.
 * @returns An array of parsed (and optionally transformed) objects.
 */
export const parseJSONL = <T = any>(
  text: string,
  transform?: (obj: any, index: number) => T,
): T[] => {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .reduce((acc: T[], line, index) => {
      try {
        const parsed = JSON.parse(line);
        acc.push(transform ? transform(parsed, index) : parsed);
      } catch (error) {
        console.error(`Failed to parse line ${index}:`, line, error);
      }
      return acc;
    }, []);
};
