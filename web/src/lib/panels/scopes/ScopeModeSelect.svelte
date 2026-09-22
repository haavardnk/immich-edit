<script lang="ts">
  import { Select } from '@immich/ui';
  import { editor } from '$lib/stores/editor.svelte';
  import { scopes, type ScopeMode } from '$lib/stores/scopes.svelte';

  const MODES: { value: ScopeMode; label: string }[] = [
    { value: 'histogram', label: 'Histogram' },
    { value: 'waveform', label: 'Waveform' },
    { value: 'parade', label: 'Parade' },
    { value: 'vectorscope', label: 'Vectorscope' }
  ];

  function select(value: ScopeMode): void {
    scopes.setMode(value);
    if (scopes.needsRender) editor.refreshScopes();
  }
</script>

<Select
  size="tiny"
  class="editor-compact-select scope-mode-select w-24 shrink-0"
  placeholder="Scope"
  options={MODES}
  value={scopes.mode}
  onChange={select}
/>
