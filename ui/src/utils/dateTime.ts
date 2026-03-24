// Format date string for min/max date inputs
export const formatDateOnly = (value: string | undefined): string => {
  if (!value) return "";
  try {
    const date = new Date(value);
    if (!isNaN(date.getTime())) {
      const year = date.getFullYear();
      const month = String(date.getMonth() + 1).padStart(2, "0");
      const day = String(date.getDate()).padStart(2, "0");
      return `${year}-${month}-${day}`;
    }
  } catch {
    // If parsing fails, return empty string
  }
  return value?.slice(0, 10) || "";
};
