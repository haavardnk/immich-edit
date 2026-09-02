export function editorHref(assetId: string, returnPath?: string | null): string {
  const path = `/assets/${encodeURIComponent(assetId)}`;
  return validReturnPath(returnPath) ? `${path}?from=${encodeURIComponent(returnPath)}` : path;
}

export function validReturnPath(path: string | null | undefined): path is string {
  return (
    typeof path === 'string' &&
    path.startsWith('/') &&
    !path.startsWith('//') &&
    !path.startsWith('/assets/')
  );
}
