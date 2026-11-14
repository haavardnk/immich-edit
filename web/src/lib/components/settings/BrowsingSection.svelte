<script lang="ts">
  import type { ClickTarget } from '#lib/api/preferences.ts';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { preferences } from '#lib/stores/preferences.svelte.ts';

  const OPTIONS: { value: ClickTarget; label: string }[] = [
    { value: 'editor', label: 'Open editor' },
    { value: 'loupe', label: 'Open loupe' }
  ];
</script>

<section class="mt-3 space-y-2 border-t border-primary/15 pt-4 pb-2">
  <h2 class="text-xs font-medium">Click a photo</h2>
  <SegmentedChoice
    options={OPTIONS}
    value={preferences.clickTarget}
    label="Click a photo"
    onChange={(target) => void preferences.setClickTarget(target)}
  />
</section>
