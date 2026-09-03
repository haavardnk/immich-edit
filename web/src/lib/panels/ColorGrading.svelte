<script lang="ts">
  import EditSlider from '$lib/components/editor/controls/EditSlider.svelte';
  import ResetButton from '$lib/components/editor/controls/ResetButton.svelte';
  import HueWheel from '$lib/components/editor/controls/HueWheel.svelte';
  import {
    segmentedControlClass,
    segmentedTabItemClass
  } from '$lib/components/editor/controls/segmentedControl';
  import Disclosure from '$lib/components/Disclosure.svelte';
  import { RadioGroup, Tabs } from 'bits-ui';
  import { editor } from '$lib/stores/editor.svelte';

  type RegionKey = 'shadows' | 'midtones' | 'highlights' | 'global';
  type Mode = 'three_way' | 'global';

  const REGION_LABELS: Record<RegionKey, string> = {
    shadows: 'Shadows',
    midtones: 'Midtones',
    highlights: 'Highlights',
    global: 'Global'
  };

  const ALL_REGIONS: RegionKey[] = ['shadows', 'midtones', 'highlights', 'global'];

  const WHEELS: { key: Exclude<RegionKey, 'global'>; size: number; cell: string }[] = [
    { key: 'midtones', size: 94, cell: 'col-span-2' },
    { key: 'shadows', size: 78, cell: '' },
    { key: 'highlights', size: 78, cell: '' }
  ];

  let mode = $state<Mode>('three_way');
  let activeRegion = $state<RegionKey>('midtones');
  let adjustOpen = $state(false);

  $effect(() => {
    if (mode === 'global') activeRegion = 'global';
    else if (activeRegion === 'global') activeRegion = 'midtones';
  });

  const activeRegionData = $derived(editor.edits.color.color_grade[activeRegion]);
  const hueGradient = 'var(--gradient-hue-linear)';
  const satGradient = $derived(
    `linear-gradient(to right, hsl(${Math.round(activeRegionData.hue)}, 0%, 50%), hsl(${Math.round(activeRegionData.hue)}, 100%, 50%))`
  );
  const lumGradient = $derived(
    `linear-gradient(to right, hsl(${Math.round(activeRegionData.hue)}, ${Math.round(activeRegionData.sat)}%, 0%), hsl(${Math.round(activeRegionData.hue)}, ${Math.round(activeRegionData.sat)}%, 50%), hsl(${Math.round(activeRegionData.hue)}, ${Math.round(activeRegionData.sat)}%, 100%))`
  );
  const balanceGradient = 'var(--gradient-grading-balance)';
  const blendingGradient = 'var(--gradient-grading-blend)';

  function resetRegion(key: RegionKey): void {
    const reg = editor.edits.color.color_grade[key];
    reg.hue = 0;
    reg.sat = 0;
    reg.lum = 0;
    void editor.onCommit(`Reset ${REGION_LABELS[key]}`);
  }

  function resetAllGrading(): void {
    const cg = editor.edits.color.color_grade;
    for (const k of ALL_REGIONS) {
      cg[k].hue = 0;
      cg[k].sat = 0;
      cg[k].lum = 0;
    }
    cg.balance = 0;
    cg.blend = 0;
    void editor.onCommit('Reset Color Grading');
  }
</script>

<div class="flex flex-col gap-1">
  <Tabs.Root
    bind:value={() => mode, (v) => (mode = v as Mode)}
    class="flex flex-col gap-1"
    activationMode="manual"
  >
    <div class="flex h-7 items-center justify-between">
      <Tabs.List class={segmentedControlClass}>
        <Tabs.Trigger value="three_way" class="{segmentedTabItemClass} px-2">3-Way</Tabs.Trigger>
        <Tabs.Trigger value="global" class="{segmentedTabItemClass} px-2">Global</Tabs.Trigger>
      </Tabs.List>
      <ResetButton title="Reset all color grading" onclick={resetAllGrading} />
    </div>

    <Tabs.Content value="three_way">
      {#if mode === 'three_way'}
        <RadioGroup.Root
          bind:value={() => activeRegion, (v) => (activeRegion = v as RegionKey)}
          orientation="horizontal"
          aria-label="Colour grading region"
          class="grid grid-cols-2 justify-items-center gap-x-2 gap-y-1"
        >
          {#each WHEELS as wheel (wheel.key)}
            <div class="flex flex-col items-center gap-0.5 {wheel.cell}">
              <div
                role="presentation"
                class="rounded-full border p-0.5 transition-[border-color,box-shadow] duration-150 {activeRegion ===
                wheel.key
                  ? 'border-primary shadow-[0_0_0_1px_var(--color-primary)]'
                  : 'border-transparent hover:border-dark/10'}"
                onpointerdowncapture={() => (activeRegion = wheel.key)}
              >
                <HueWheel
                  bind:hue={editor.edits.color.color_grade[wheel.key].hue}
                  bind:sat={editor.edits.color.color_grade[wheel.key].sat}
                  size={wheel.size}
                  onLive={editor.onLive}
                  onCommit={editor.onCommit}
                  commitAction="{REGION_LABELS[wheel.key]} Color"
                />
              </div>
              <RadioGroup.Item
                value={wheel.key}
                class="h-5 px-1.5 text-[10px] font-medium text-dark/65 transition-colors duration-150 hover:text-dark aria-checked:text-primary"
              >
                {REGION_LABELS[wheel.key]}
              </RadioGroup.Item>
            </div>
          {/each}
        </RadioGroup.Root>
      {/if}
    </Tabs.Content>
    <Tabs.Content value="global">
      {#if mode === 'global'}
        <div class="flex flex-col items-center gap-1 py-1">
          <HueWheel
            bind:hue={editor.edits.color.color_grade.global.hue}
            bind:sat={editor.edits.color.color_grade.global.sat}
            size={130}
            onLive={editor.onLive}
            onCommit={editor.onCommit}
            commitAction="Global Color"
          />
          <div class="text-[10px] font-medium text-dark/65">Global</div>
        </div>
      {/if}
    </Tabs.Content>
  </Tabs.Root>

  <div class="flex flex-col gap-1 border-t border-hairline pt-1">
    <Disclosure
      open={adjustOpen}
      title="Adjust — {REGION_LABELS[activeRegion]}"
      variant="inline"
      onOpenChange={(v) => (adjustOpen = v)}
    >
      <div class="flex flex-col gap-1 pt-1">
        <div
          class="flex items-center justify-between text-[10px] font-mono tabular-nums text-dark/65 px-1"
        >
          <span>H: {Math.round(activeRegionData.hue)}</span>
          <span>S: {Math.round(activeRegionData.sat)}</span>
          <ResetButton
            title="Reset {REGION_LABELS[activeRegion]}"
            onclick={() => resetRegion(activeRegion)}
          />
        </div>
        <EditSlider
          label="Hue"
          commitAction={`${REGION_LABELS[activeRegion]} Hue`}
          bind:value={activeRegionData.hue}
          min={0}
          max={360}
          gradient={hueGradient}
        />
        <EditSlider
          label="Saturation"
          commitAction={`${REGION_LABELS[activeRegion]} Saturation`}
          bind:value={activeRegionData.sat}
          min={0}
          max={100}
          gradient={satGradient}
        />
        <EditSlider
          label="Luminance"
          commitAction={`${REGION_LABELS[activeRegion]} Luminance`}
          bind:value={activeRegionData.lum}
          min={-50}
          max={50}
          gradient={lumGradient}
        />
      </div>
    </Disclosure>
  </div>

  <div class="flex flex-col gap-1 border-t border-hairline pt-1">
    <EditSlider
      label="Balance"
      commitAction="Color Balance"
      bind:value={editor.edits.color.color_grade.balance}
      min={-100}
      max={100}
      gradient={balanceGradient}
    />
    <EditSlider
      label="Blending"
      commitAction="Color Blending"
      bind:value={editor.edits.color.color_grade.blend}
      min={0}
      max={100}
      gradient={blendingGradient}
    />
  </div>
</div>
