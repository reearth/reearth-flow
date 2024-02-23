import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

// Needed for shadcn/ui
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
