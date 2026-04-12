import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };

/**
 * Basic input sanitization to prevent command injection or malformed strings.
 * Filters out characters that could be problematic in a shell environment.
 */
export function sanitize(input: string): string {
  // Allow alphanumeric, underscores, hyphens, dots, colons (for tags), slashes, and spaces
  // Remove shell metacharacters: ; & | $ ` " ' < > ( ) [ ] { } * ? ! \
  return input.replace(/[;&|$`"'<>()[\]{}*?!\\ ]/g, '');
}

/**
 * Specifically for image names and tags, which should not have spaces.
 */
export function sanitizeImageName(input: string): string {
    return input.replace(/[;&|$`"'<>()[\]{}*?!\\ ]/g, '');
}

/**
 * For paths and other inputs where spaces might be valid but shell meta-chars are not.
 */
export function sanitizePath(input: string): string {
    // Remove shell metacharacters but allow spaces, dots, slashes, etc.
    return input.replace(/[;&|$`"'<>()[\]{}*?!\\ ]/g, (match) => {
        if (match === ' ') return ' ';
        return '';
    });
}

/**
 * Sanitizes a port mapping string (e.g. "8080:80").
 * Only allows numbers and colons.
 */
export function sanitizePorts(ports: string): string {
  // Allow numbers and colons, plus comma for multiple mappings
  return ports.replace(/[^0-9:,]/g, '');
}

/**
 * Parses a human-readable size string (e.g., "1.2GB", "500MB") into bytes.
 */
export function parseSize(sizeStr: string): number {
  const match = sizeStr.match(/^(\d+(?:\.\d+)?)\s*([a-zA-Z]+)$/);
  if (!match) return 0;
  const value = parseFloat(match[1]);
  const unit = match[2].toUpperCase();
  const units: Record<string, number> = {
    'B': 1,
    'KB': 1024,
    'MB': 1024 * 1024,
    'GB': 1024 * 1024 * 1024,
    'TB': 1024 * 1024 * 1024 * 1024,
    'KIB': 1024,
    'MIB': 1024 * 1024,
    'GIB': 1024 * 1024 * 1024,
    'TIB': 1024 * 1024 * 1024 * 1024,
  };
  return value * (units[unit] || 1);
}
