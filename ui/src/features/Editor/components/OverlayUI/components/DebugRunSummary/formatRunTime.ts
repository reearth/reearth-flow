import { isSameDay, isSameYear } from "date-fns";

// When a run started or finished, short enough for one line: the time alone
// for today, with the date for any other day, and with the year for another
// year. Seeing the date is what tells a run left from a previous day apart
// from one just made. Returns undefined for a missing or unreadable input.
export const formatRunTime = (
  input: string | undefined,
  lang?: string,
  now: Date = new Date(),
): string | undefined => {
  if (!input) return undefined;
  const date = new Date(input);
  if (Number.isNaN(date.getTime())) return undefined;

  const time: Intl.DateTimeFormatOptions = {
    hour: "2-digit",
    minute: "2-digit",
  };
  const options: Intl.DateTimeFormatOptions = isSameDay(date, now)
    ? time
    : isSameYear(date, now)
      ? { ...time, month: "short", day: "numeric" }
      : { ...time, year: "numeric", month: "short", day: "numeric" };

  try {
    return new Intl.DateTimeFormat(lang, options).format(date);
  } catch {
    // An unknown locale tag throws; fall back to the browser's own.
    return new Intl.DateTimeFormat(undefined, options).format(date);
  }
};
