<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
</script>

{#if exportSettings.destination === 'download'}
  <CheckboxRow
    label="Save automatically"
    class="mt-1 touch:h-11 touch:text-sm"
    checked={exportSettings.saveAutomatically}
    onChange={(checked) => (exportSettings.saveAutomatically = checked)}
  >
    <span class="ms-1 text-dark/50">on this device</span>
  </CheckboxRow>
{/if}
