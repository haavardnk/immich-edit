<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import type { ColorSpaceOpt } from '$lib/api/export';
  import { Field, Select, Text } from '@immich/ui';
  const wideGamut = $derived(
    typeof window !== 'undefined' && window.matchMedia('(color-gamut: p3)').matches
  );
  const spaces = $derived([
    { value: 'srgb', label: 'sRGB' },
    { value: 'displayp3', label: 'Display P3', disabled: !wideGamut }
  ]);
</script>

<div class="flex flex-col gap-2 border-t border-dark/10 pt-2">
  <Text size="tiny" color="muted" fontWeight="medium">Soft proof</Text>
  <div class="flex flex-col gap-2.5">
    <Field label="Proof space" size="tiny">
      <Select
        size="tiny"
        options={spaces}
        value={editor.proofSpace}
        onChange={(v) => editor.setProofSpace(v as ColorSpaceOpt)}
      />
      {#if !wideGamut}
        <span class="text-[10px] leading-tight text-dark/65">
          Display is not wide-gamut; P3 proof unavailable.
        </span>
      {/if}
    </Field>
    <CheckboxRow
      label="Show gamut warning"
      class="text-xs"
      checked={editor.gamutWarn}
      onChange={editor.toggleGamutWarn}
    />
  </div>
</div>
