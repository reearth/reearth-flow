import { intlFormatDistance } from "date-fns";

export const formatDate = (input: string | Date) => {
  const date = new Date(input);
  return `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()} ${date.getHours()}:${date.getMinutes() < 10 ? "0" + date.getMinutes() : date.getMinutes()}`;
};

export const formatRelativeTime = (
  input: string | Date,
  lang?: string,
  now: Date = new Date(),
) => intlFormatDistance(new Date(input), now, { locale: lang });
