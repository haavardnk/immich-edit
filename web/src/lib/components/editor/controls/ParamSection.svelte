<script lang="ts">
  import ParamSlider from './ParamSlider.svelte';
  import SectionHeader from './SectionHeader.svelte';
  import { commitSectionReset } from './paramReset';
  import { paramsModified } from '#lib/editor/params/define.ts';
  import type { SectionDef } from '#lib/editor/params/types.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';

  let { section }: { section: SectionDef } = $props();

  const params = $derived(section.rows.flat());
  const modified = $derived(paramsModified(editor.edits, params, editor.isRaw));
</script>

<SectionHeader
  title={section.title}
  section={section.id}
  {modified}
  onReset={() => commitSectionReset(section)}
/>
{#each section.rows as row, index (index)}
  {#if index > 0}
    <div class="border-t border-dark/10"></div>
  {/if}
  {#each row as param (param.id)}
    <ParamSlider {param} />
  {/each}
{/each}
