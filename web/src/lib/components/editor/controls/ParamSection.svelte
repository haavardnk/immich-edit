<script lang="ts">
  import ParamSlider from './ParamSlider.svelte';
  import SectionHeader from './SectionHeader.svelte';
  import { commitParamReset, commitSectionReset } from './paramReset';
  import { paramsModified, sectionLabel } from '#lib/editor/params/define.ts';
  import type { ParamDef, SectionDef } from '#lib/editor/params/types.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';

  let { section, params: subset }: { section: SectionDef; params?: readonly ParamDef[] } = $props();

  const rows = $derived(subset ? [subset] : section.rows);
  const params = $derived(rows.flat());
  const modified = $derived(paramsModified(editor.edits, params, editor.isRaw));

  function reset(): void {
    if (subset) commitParamReset(subset, `Reset ${sectionLabel(section)}`);
    else commitSectionReset(section);
  }
</script>

<SectionHeader title={section.title} section={section.id} {modified} onReset={reset} />
{#each rows as row, index (index)}
  {#if index > 0}
    <div class="border-t border-dark/10"></div>
  {/if}
  {#each row as param (param.id)}
    <ParamSlider {param} />
  {/each}
{/each}
