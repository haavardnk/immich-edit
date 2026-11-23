const DOCS_ORIGIN = 'https://immich-edit.iridiumlab.dev';

export function docsUrl(path = '', version = __APP_VERSION__): string {
  const segment =
    !version || version === '0.0.0' ? 'main' : version.startsWith('v') ? version : `v${version}`;
  const clean = path.replace(/^\/+/, '');
  return clean ? `${DOCS_ORIGIN}/${segment}/${clean}` : `${DOCS_ORIGIN}/${segment}/`;
}
