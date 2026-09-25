import { addTagToAsset, removeTagFromAsset } from '$lib/api/tags';
import { ensureManagedTag, MANAGED_TAG_PREFIX } from '$lib/managedTags';
import type { TagRef } from '$lib/types/asset';

export const LABEL_COLORS = ['red', 'yellow', 'green', 'blue', 'purple'] as const;

export type LabelColor = (typeof LABEL_COLORS)[number];

const LABEL_PREFIX = `${MANAGED_TAG_PREFIX}/label/`;

export const LABEL_NAMES: Record<LabelColor, string> = {
  red: 'Red',
  yellow: 'Yellow',
  green: 'Green',
  blue: 'Blue',
  purple: 'Purple'
};

export const LABEL_TEXT: Record<LabelColor, string> = {
  red: 'text-red-500',
  yellow: 'text-yellow-400',
  green: 'text-green-500',
  blue: 'text-blue-500',
  purple: 'text-purple-500'
};

const LABEL_KEYS: Record<string, LabelColor> = {
  '6': 'red',
  '7': 'yellow',
  '8': 'green',
  '9': 'blue'
};

export function labelKey(color: LabelColor): string | null {
  return Object.keys(LABEL_KEYS).find((key) => LABEL_KEYS[key] === color) ?? null;
}

export function labelTagValue(color: LabelColor): string {
  return `${LABEL_PREFIX}${color}`;
}

function labelColorOf(tag: TagRef): LabelColor | null {
  const value = tag.value?.toLowerCase();
  if (!value?.startsWith(LABEL_PREFIX)) return null;
  return LABEL_COLORS.find((c) => c === value.slice(LABEL_PREFIX.length)) ?? null;
}

export function isLabelTag(tag: TagRef): boolean {
  return labelColorOf(tag) !== null;
}

export function labelOf(asset: { tags?: TagRef[] | null }): LabelColor | null {
  for (const tag of asset.tags ?? []) {
    const color = labelColorOf(tag);
    if (color) return color;
  }
  return null;
}

export function withLabel(tags: TagRef[], tag: TagRef | null): TagRef[] {
  const kept = tags.filter((t) => !isLabelTag(t));
  return tag ? [...kept, tag] : kept;
}

export function nextLabelFromKey(
  key: string,
  current: LabelColor | null
): LabelColor | null | undefined {
  const color = LABEL_KEYS[key];
  if (!color) return undefined;
  return color === current ? null : color;
}

export async function labelTagFor(color: LabelColor | null): Promise<TagRef | null | undefined> {
  if (!color) return null;
  return (await ensureManagedTag(labelTagValue(color))) ?? undefined;
}

export async function writeLabel(
  assetId: string,
  tags: TagRef[],
  tag: TagRef | null
): Promise<void> {
  for (const old of tags.filter(isLabelTag)) {
    if (old.id !== tag?.id) await removeTagFromAsset(old.id, assetId);
  }
  if (tag && !tags.some((t) => t.id === tag.id)) await addTagToAsset(tag.id, assetId);
}
