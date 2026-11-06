<script lang="ts">
  import { NOT_OWNER, ownsAsset } from '#lib/browse/ownership.ts';
  import InfoSection from '#lib/components/browse/info/InfoSection.svelte';
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import TagPicker from '#lib/components/TagPicker.svelte';
  import { loupeTags } from '#lib/stores/loupeTags.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';

  let { asset }: { asset: AssetSummary } = $props();

  let picking = $state(false);

  const owned = $derived(ownsAsset(asset));
  const tags = $derived(loupeTags.tagsOf(asset.id));
</script>

<InfoSection title="Tags">
  <DisabledReason locked={!owned} reason={NOT_OWNER} class="flex min-w-0">
    <TagPicker
      {tags}
      disabled={!owned}
      open={picking}
      onToggle={() => (picking = !picking)}
      onClose={() => (picking = false)}
      onAdd={(tag) => loupeTags.add(asset.id, tag)}
      onRemove={(tagId) => loupeTags.remove(asset.id, tagId)}
      onCreate={(value) => loupeTags.createAndAdd(asset.id, value)}
      anchor="bottom"
      align="end"
      addLabel="Add"
      subtitle={asset.originalFileName}
      wrap
    />
  </DisabledReason>
</InfoSection>
