<script lang="ts">
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import ResetButton from '#lib/components/editor/controls/ResetButton.svelte';
  import {
    commitParamReset,
    commitSectionReset
  } from '#lib/components/editor/controls/paramReset.ts';
  import HueWheel from '#lib/components/editor/controls/HueWheel.svelte';
  import {
    segmentedControlClass,
    segmentedTabItemClass
  } from '#lib/components/editor/controls/segmentedControl.ts';
  import Disclosure from '#lib/components/Disclosure.svelte';
  import { RadioGroup, Tabs } from 'bits-ui';
  import {
    COLOR_GRADING,
    GRADE_REGION_LABELS,
    GRADE_REGIONS,
    gradeBalance,
    gradeBlend,
    type GradeRegion
  } from '#lib/editor/params/colorGrading.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';

  type Mode = 'three_way' | 'global';

  const WHEELS: { key: Exclude<GradeRegion, 'global'>; size: number; cell: string }[] = [
    { key: 'midtones', size: 94, cell: 'col-span-2' },
    { key: 'shadows', size: 78, cell: '' },
    { key: 'highlights', size: 78, cell: '' }
  ];

  let mode = $state<Mode>('three_way');
  let activeRegion = $state<GradeRegion>('midtones');
  let adjustOpen = $state(false);

  $effect(() => {
    if (mode === 'global') activeRegion = 'global';
    else if (activeRegion === 'global') activeRegion = 'midtones';
  });

  const activeRegionData = $derived(editor.edits.color.color_grade[activeRegion]);
  const activeLabel = $derived(GRADE_REGION_LABELS[activeRegion]);
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
      <ResetButton
        title="Reset all color grading"
        onclick={() => commitSectionReset(COLOR_GRADING)}
      />
    </div>

    <Tabs.Content value="three_way">
      {#if mode === 'three_way'}
        <RadioGroup.Root
          bind:value={() => activeRegion, (v) => (activeRegion = v as GradeRegion)}
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
                  commitAction="{GRADE_REGION_LABELS[wheel.key]} Color"
                />
              </div>
              <RadioGroup.Item
                value={wheel.key}
                class="h-5 px-1.5 text-[10px] font-medium text-dark/65 transition-colors duration-150 hover:text-dark aria-checked:text-primary"
              >
                {GRADE_REGION_LABELS[wheel.key]}
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
      title="Adjust — {activeLabel}"
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
            title="Reset {activeLabel}"
            onclick={() => commitParamReset(GRADE_REGIONS[activeRegion], `Reset ${activeLabel}`)}
          />
        </div>
        {#each GRADE_REGIONS[activeRegion] as param (param.id)}
          <ParamSlider {param} />
        {/each}
      </div>
    </Disclosure>
  </div>

  <div class="flex flex-col gap-1 border-t border-hairline pt-1">
    <ParamSlider param={gradeBalance} />
    <ParamSlider param={gradeBlend} />
  </div>
</div>
