import type { InfoSource } from '#lib/browse/info/infoSource.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { editorTarget } from '#lib/stores/editor/metadata.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export const EDITOR_INFO: InfoSource = {
  target: () => editorTarget(editor),
  tagsOf: () => editor.asset?.tags ?? [],
  addTag: (_, tag) => editor.addTag(tag),
  removeTag: (_, tagId) => editor.removeTag(tagId),
  createTag: (_, value) => editor.createAndAddTag(value)
};

export function editorInfoAsset(): AssetSummary | null {
  return editor.asset ?? (editor.assetId ? browsing.find(editor.assetId) : null);
}
