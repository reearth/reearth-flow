import { parseJSONL } from "./jsonl";
import { intermediateDataTransform } from "./jsonl/transformIntermediateData";

export type SupportedDataTypes = "geojson";

export async function fetchAndReadData(fileUrl: string): Promise<{
  fileContent: any;
  type: SupportedDataTypes | null;
  error: string | null;
}> {
  if (!fileUrl.trim()) {
    return { fileContent: null, type: null, error: "Please enter a URL" };
  }

  try {
    const response = await fetch(fileUrl);
    console.log(" MY RESPONSE", response);

    if (!response.ok) {
      throw new Error(
        `Failed to fetch: ${response.status} ${response.statusText}`,
      );
    }

    const fileExtension = fileUrl.split(".").pop()?.toLowerCase();
    if (fileExtension === "geojson") {
      const content = await response.text();
      const parsedData = JSON.parse(content);
      return { fileContent: parsedData, type: "geojson", error: null };
    } else if (fileExtension === "jsonl") {
      const text = await response.text();
      const parsedJSONL = parseJSONL(text, {
        transform: intermediateDataTransform,
      });
      const interData = {
        type: "FeatureCollection",
        features: parsedJSONL,
      };
      return {
        fileContent: interData,
        type: "geojson",
        error: null,
      };
    } else {
      console.log("Unsupported file format");
      return {
        fileContent: null,
        type: null,
        error: "File format not supported",
      };
    }
  } catch (err) {
    return {
      fileContent: null,
      type: null,
      error: `Error fetching file: ${err instanceof Error ? err.message : String(err)}`,
    };
  }
}
