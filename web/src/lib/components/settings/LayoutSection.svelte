<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { layoutStatus } from '#lib/device/status.ts';
  import { LAYOUT_CHOICES, type LayoutChoice } from '#lib/device/layoutRules.ts';

  const LABELS: Record<LayoutChoice, string> = {
    auto: 'Auto',
    desktop: 'Desktop',
    touch: 'Touch'
  };

  const OPTIONS = LAYOUT_CHOICES.map((value) => ({ value, label: LABELS[value] }));

  const status = $derived(layoutStatus(layout.choice, layout.mode));
</script>

<section class="space-y-2 py-2">
  <SegmentedChoice
    options={OPTIONS}
    value={layout.choice}
    label="Layout"
    onChange={(choice) => layout.setChoice(choice)}
    {status}
  />
</section>
