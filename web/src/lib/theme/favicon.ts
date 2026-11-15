export const FAVICON_ACCENT = '#B7A1FB';

export type FaviconBadge = { kind: 'progress'; fraction: number } | { kind: 'alert' };

const BRACKETS = 'M64 182V80h102M346 80h102v102M448 330v102H346M166 432H64V330';
const RING_RADIUS = 216;
const RING_LENGTH = 2 * Math.PI * RING_RADIUS;
const ALERT_RED = '#E5484D';

function frame(accent: string, badge: FaviconBadge | undefined): string[] {
  if (badge?.kind !== 'progress') {
    return [
      `  <path d="${BRACKETS}" fill="none" stroke="${accent}" stroke-width="38" stroke-linecap="round" stroke-linejoin="round" />`
    ];
  }
  const filled = (Math.min(1, Math.max(0, badge.fraction)) * RING_LENGTH).toFixed(1);
  const ring = `cx="256" cy="256" r="${RING_RADIUS}" fill="none" stroke="${accent}" stroke-width="40"`;
  return [
    `  <circle ${ring} stroke-opacity="0.3" />`,
    `  <circle ${ring} stroke-linecap="round" stroke-dasharray="${filled} ${RING_LENGTH.toFixed(1)}" transform="rotate(-90 256 256)" />`
  ];
}

export function faviconSvg(accent: string, badge?: FaviconBadge): string {
  return [
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512" role="img" aria-label="immich-edit">',
    '  <style>',
    '    .ink { fill: #27272A; }',
    '    @media (prefers-color-scheme: dark) { .ink { fill: #E5E7EB; } }',
    '  </style>',
    ...frame(accent, badge),
    '  <rect class="ink" x="150" y="144" width="52" height="224" rx="26" />',
    '  <path class="ink" d="M262 144h116v42h-68v48h58v42h-58v50h68v42H262Z" />',
    ...(badge?.kind === 'alert'
      ? [`  <circle cx="424" cy="88" r="88" fill="${ALERT_RED}" />`]
      : []),
    '</svg>',
    ''
  ].join('\n');
}

export function faviconDataUrl(accent: string, badge?: FaviconBadge): string {
  return `data:image/svg+xml,${encodeURIComponent(faviconSvg(accent, badge))}`;
}
