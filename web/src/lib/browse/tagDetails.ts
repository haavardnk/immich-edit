import type { TagPatch, TagSummary } from '#lib/api/tags.ts';
import { MANAGED_TAG_PREFIX } from '#lib/browse/managedTags.ts';
import { sameTagColor } from '#lib/browse/tagColor.ts';
import { splitTagValue, tagPathLabel } from '#lib/browse/tagTree.ts';
import { DISPLAY_NAME_MAX, displayNameTooLong } from '#lib/utils/displayName.ts';

export function siblingTags(tags: readonly TagSummary[], parentValue: string): TagSummary[] {
  return tags.filter((tag) => splitTagValue(tag.value).parent === parentValue);
}

export function tagNameProblem(
  name: string,
  siblings: readonly TagSummary[],
  parentValue: string,
  selfId: string | null
): string | null {
  const trimmed = name.trim();
  if (trimmed === '') return null;
  if (trimmed.includes('/')) return 'Tag names cannot contain /';
  if (/\p{Cc}/u.test(trimmed)) return 'Tag names cannot contain control characters';
  if (displayNameTooLong(trimmed)) return `Use at most ${DISPLAY_NAME_MAX} characters`;
  if (parentValue === '' && trimmed.toLowerCase() === MANAGED_TAG_PREFIX) {
    return `${MANAGED_TAG_PREFIX} is kept for immich-edit's own tags`;
  }
  const folded = trimmed.toLowerCase();
  const clash = siblings.find((tag) => tag.id !== selfId && tag.name.toLowerCase() === folded);
  return clash ? `${tagPathLabel(clash.value)} already exists` : null;
}

export function tagDetailsPatch(
  tag: TagSummary,
  name: string,
  color: string | null,
  canRename: boolean
): TagPatch | null {
  const patch: TagPatch = {};
  const trimmed = name.trim();
  if (canRename && trimmed !== tag.name) patch.name = trimmed;
  if (!sameTagColor(color, tag.color)) patch.color = color;
  return Object.keys(patch).length > 0 ? patch : null;
}

export function createdTagPath(parentValue: string, name: string): string {
  const trimmed = name.trim();
  return tagPathLabel(parentValue === '' ? trimmed : `${parentValue}/${trimmed}`);
}
