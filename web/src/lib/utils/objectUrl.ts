export function makeObjectUrl(blob: Blob): string {
  return URL.createObjectURL(blob);
}

export function revoke(url: string | null): void {
  if (url) URL.revokeObjectURL(url);
}
