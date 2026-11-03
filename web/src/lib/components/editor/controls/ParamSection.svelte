<script lang="ts">
  import ParamSlider from './ParamSlider.svelte';
  import SectionHeader from './SectionHeader.svelte';
  import type { SectionDef } from '#lib/editor/params/types.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';

  let { section }: { section: SectionDef } = $props();

  const params = $derived(section.rows.flat());
  const modified = $derived(params.some((param) => param.get(editor.edits) !== param.defaultValue));

  function reset(): void {
    for (const param of params) param.set(editor.edits, param.defaultValue);
    void editor.onCommit(`Reset ${section.title}`);
  }
</script>

<SectionHeader title={section.title} section={section.id} {modified} onReset={reset} />
{#each section.rows as row, index (index)}
  {#if index > 0}
    <div class="border-t border-dark/10"></div>
  {/if}
  {#each row as param (param.id)}
    <ParamSlider {param} />
  {/each}
{/each}
