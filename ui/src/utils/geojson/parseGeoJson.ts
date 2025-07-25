export function parseGeoJson(content: string) {
  if (!content) return null;
  const parsedData = JSON.parse(content);

  return {
    ...parsedData,
    features: parsedData.features.map((f: any) => ({
      ...f,
      properties: {
        _originalId: f.id,
        ...f.properties,
      },
    })),
  };
}
