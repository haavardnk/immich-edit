<script lang="ts">
  import type { ExportForm } from './settings';

  let { form = $bindable<ExportForm>() }: { form: ExportForm } = $props();
</script>

<div class="flex flex-col gap-1">
  <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Filename suffix</span>
  <input
    type="text"
    class="input w-full bg-immich-dark-bg/40 border-immich-dark-fg/10 text-xs h-auto py-2.5 min-h-0"
    bind:value={form.filenameSuffix}
    placeholder="_edit"
  />
</div>
