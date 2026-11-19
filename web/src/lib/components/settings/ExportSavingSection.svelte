<script lang="ts">
  import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
  import { Switch } from '@immich/ui';

  const id = $props.id();
</script>

<section class="py-2">
  <div class="flex items-center gap-3 text-xs">
    <div class="min-w-0 flex-1">
      <div id="{id}-label">Save automatically</div>
      <div id="{id}-hint" class="text-dark/65">Save each export as soon as it renders.</div>
    </div>
    <Switch
      checked={exportSettings.saveAutomatically}
      aria-labelledby="{id}-label"
      aria-describedby="{id}-hint"
      onCheckedChange={(checked) => (exportSettings.saveAutomatically = checked)}
    />
  </div>
</section>
