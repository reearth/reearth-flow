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
    } else {
      console.log("didn't get geojson");
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
