import type { MaskComponent, MaskLayer } from '#lib/types/edits.ts';
import { generatedLabel, kindLabel, numberRepeats } from '#lib/types/masks.ts';
import { countOf } from '#lib/utils/plural.ts';

function decimal(value: number): string {
  return value.toFixed(2);
}

export function shapeSubtitle(comp: MaskComponent): string {
  if (!comp.enabled) return 'Hidden';
  const parts: string[] = [];
  const generated = comp.generated;
  if (generated) {
    parts.push('AI');
    if (generated.points?.length) parts.push(countOf(generated.points.length, 'point'));
    if (generated.grow !== 0) parts.push(`Grow ${Math.round(generated.grow)}`);
    if (generated.feather !== 0) parts.push(`Feather ${Math.round(generated.feather)}`);
    if (generated.painted) parts.push('Painted');
  } else {
    const kind = comp.kind;
    if (kind.kind === 'linear' || kind.kind === 'radial')
      parts.push(`Feather ${decimal(kind.feather)}`);
    else if (kind.kind === 'polygon') parts.push(countOf(kind.points.length, 'corner'));
    else if (kind.kind === 'luma_range') parts.push(`${decimal(kind.min)} to ${decimal(kind.max)}`);
    else if (kind.kind === 'color_range') parts.push(`Tolerance ${decimal(kind.tolerance)}`);
    else parts.push('Painted');
  }
  if (comp.invert) parts.push('Inverted');
  return parts.join(' · ');
}

function generatedName(kind: string, maskClass: string | undefined): string {
  const label = generatedLabel(kind);
  return maskClass ? `${label} · ${maskClass}` : label;
}

export function shapeLabels(layer: MaskLayer | null): string[] {
  if (!layer) return [];
  return numberRepeats(
    layer.components.map((c) =>
      c.generated ? generatedName(c.generated.kind, c.generated.class) : kindLabel(c.kind)
    )
  );
}
