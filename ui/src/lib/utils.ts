import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

// Needed for shadcn/ui
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function isDefined<T>(argument: T | undefined | null): argument is T {
  return argument !== undefined || argument !== null;
}
