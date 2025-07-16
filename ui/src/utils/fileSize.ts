export const formatFileSize = (sizeInBytes: number): string => {
  if (isNaN(sizeInBytes) || sizeInBytes < 0) return "Invalid size";

  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = sizeInBytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
};
