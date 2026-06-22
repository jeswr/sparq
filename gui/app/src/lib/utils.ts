import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/** Merge Tailwind class lists with conflict resolution (the shadcn `cn` convention). */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
