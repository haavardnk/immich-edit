import { labelTagValue, type LabelColor } from '$lib/labels';
import type { TagRef } from '$lib/types/asset';
import { TagMembers } from './tagMembers.svelte';

export const labelMembers: Record<LabelColor, TagMembers> = {
  red: new TagMembers(labelTagValue('red')),
  yellow: new TagMembers(labelTagValue('yellow')),
  green: new TagMembers(labelTagValue('green')),
  blue: new TagMembers(labelTagValue('blue')),
  purple: new TagMembers(labelTagValue('purple'))
};

export function assignLabel(id: string, color: LabelColor | null, tag: TagRef | null): void {
  for (const members of Object.values(labelMembers)) members.remove(id);
  if (color && tag) labelMembers[color].add(id, tag);
}
