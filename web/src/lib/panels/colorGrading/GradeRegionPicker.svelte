<script lang="ts">
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import { commitParamReset } from '#lib/components/editor/controls/paramReset.ts';
  import HueWheel from '#lib/components/editor/controls/HueWheel.svelte';
  import {
    GRADE_REGION_LABELS,
    GRADE_REGIONS,
    type GradeRegion
  } from '#lib/editor/params/colorGrading.ts';
  import { paramsModified } from '#lib/editor/params/define.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { settingSearch } from '#lib/stores/settingSearch.svelte.ts';
  import { Button } from '@immich/ui';
  import { mdiRestore } from '@mdi/js';
  import { RadioGroup } from 'bits-ui';

  const REGIONS = Object.keys(GRADE_REGIONS) as GradeRegion[];
  const WHEEL_SIZE = 168;

  let region = $state<GradeRegion>('midtones');

  $effect(() => {
    const revealed = REGIONS[settingSearch.revealedIn(Object.values(GRADE_REGIONS))];
    if (revealed) region = revealed;
  });

  const label = $derived(GRADE_REGION_LABELS[region]);
  const params = $derived(GRADE_REGIONS[region]);
  const data = $derived(editor.edits.color.color_grade[region]);
  const luminance = $derived(params.find((param) => param.id === `grade_${region}_lum`));
  const modified = $derived(
    new Set(REGIONS.filter((key) => paramsModified(editor.edits, GRADE_REGIONS[key], editor.isRaw)))
  );
</script>

<div class="flex flex-col gap-2">
  <RadioGroup.Root
    bind:value={() => region, (v) => (region = v as GradeRegion)}
    orientation="horizontal"
    aria-label="Colour grading region"
    class="-mx-3 flex gap-1.5 overflow-x-auto px-3 py-1 scrollbar-hidden"
  >
    {#each REGIONS as key (key)}
      <RadioGroup.Item
        value={key}
        class="relative flex h-8 shrink-0 items-center gap-1.5 rounded-full px-3 text-[13.5px] whitespace-nowrap outline-none before:absolute before:inset-x-0 before:-inset-y-1.5 focus-visible:ring-2 focus-visible:ring-primary aria-checked:bg-primary/22 aria-checked:text-light-950 aria-[checked=false]:bg-ghost aria-[checked=false]:text-dark"
      >
        {GRADE_REGION_LABELS[key]}
        {#if modified.has(key)}
          <span data-modified-indicator aria-hidden="true" class="size-1.5 rounded-full bg-primary"
          ></span>
        {/if}
      </RadioGroup.Item>
    {/each}
  </RadioGroup.Root>

  <div class="flex items-center gap-4">
    <HueWheel
      bind:hue={editor.edits.color.color_grade[region].hue}
      bind:sat={editor.edits.color.color_grade[region].sat}
      size={WHEEL_SIZE}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      commitAction="{label} Color"
    />
    <div class="flex min-w-0 flex-col gap-1.5 text-[13px] text-dark/65">
      <div>
        Hue <span class="font-mono text-light-950 tabular-nums">{Math.round(data.hue)}°</span>
      </div>
      <div>
        Saturation <span class="font-mono text-light-950 tabular-nums">{Math.round(data.sat)}</span>
      </div>
      <Button
        size="small"
        variant="ghost"
        color="secondary"
        leadingIcon={mdiRestore}
        class="mt-1 h-9 self-start rounded-full bg-ghost px-3.5 text-[13px]"
        aria-label="Reset {label}"
        disabled={!modified.has(region)}
        onclick={() => commitParamReset(params, `Reset ${label}`)}
      >
        {label}
      </Button>
    </div>
  </div>

  {#if luminance}
    <ParamSlider param={luminance} />
  {/if}
</div>
