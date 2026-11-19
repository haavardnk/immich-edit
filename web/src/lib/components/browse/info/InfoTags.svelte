<script lang="ts">
  import { NOT_OWNER, ownsAsset } from '#lib/browse/ownership.ts';
  import InfoSection from '#lib/components/browse/info/InfoSection.svelte';
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import TagPicker from '#lib/components/TagPicker.svelte';
  import { infoSource } from '#lib/browse/info/infoSource.ts';
  import type { AssetSummary } from '#lib/types/album.ts';

  let { asset }: { asset: AssetSummary } = $props();

  let picking = $state(false);

  const owned = $derived(ownsAsset(asset));
  const source = infoSource();
  const tags = $derived(source.tagsOf(asset.id));
</script>

<InfoSection title="Tags">
  <DisabledReason locked={!owned} reason={NOT_OWNER} class="flex min-w-0">
    <TagPicker
      {tags}
      disabled={!owned}
      open={picking}
      onToggle={() => (picking = !picking)}
      onClose={() => (picking = false)}
      onAdd={(tag) => void source.addTag(asset.id, tag)}
      onRemove={(tagId) => void source.removeTag(asset.id, tagId)}
      onCreate={(value) => source.createTag(asset.id, value)}
      anchor="bottom"
      align="end"
      addLabel="Add"
      subtitle={asset.originalFileName}
      wrap
    />
  </DisabledReason>
</InfoSection>
