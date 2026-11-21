import type { TagSummary } from '#lib/api/tags.ts';
import type { DialogText, DialogTextPart } from '#lib/browse/dialogText.ts';
import { splitTagValue, tagPathLabel } from '#lib/browse/tagTree.ts';
import { countOf } from '#lib/utils/plural.ts';

function photosLine(photos: number, subtags: number): string {
  if (photos === 0) return 'No photos have it.';
  const subject = subtags > 0 ? 'They come' : 'It comes';
  return `${subject} off ${countOf(photos, 'photo')}. The photos stay in your library with their other tags.`;
}

export function deleteTagText(tag: TagSummary, subtags: number): DialogText {
  const branch = subtags > 0 ? ` and its ${countOf(subtags, 'sub-tag')}.` : '.';
  const lines: DialogTextPart[][] = [
    ['Immich deletes the tag ', { strong: tagPathLabel(tag.value) }, branch],
    [photosLine(tag.assetCount ?? 0, subtags)],
    ['This cannot be undone.']
  ];
  return { title: 'Delete tag', confirm: 'Delete tag', lines };
}

export function tagCaption(tag: TagSummary, subtags: number): string {
  const parent = splitTagValue(tag.value).parent;
  return [
    parent ? tagPathLabel(parent) : null,
    tag.assetCount == null ? null : countOf(tag.assetCount, 'photo'),
    subtags > 0 ? countOf(subtags, 'sub-tag') : null
  ]
    .filter((part) => part !== null)
    .join(' · ');
}
