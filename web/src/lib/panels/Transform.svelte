<script lang="ts">
  import CropSection from '#lib/panels/geometry/CropSection.svelte';
  import TransformSection from '#lib/panels/geometry/TransformSection.svelte';
  import { useGeometrySession } from '#lib/panels/geometry/geometrySession.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';

  useGeometrySession();
</script>

<div class="flex flex-col divide-y divide-dark/10">
  {#if editor.geometrySession}
    <CropSection sess={editor.geometrySession} />
    <TransformSection sess={editor.geometrySession} />
  {/if}
</div>
