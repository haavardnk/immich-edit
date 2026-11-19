<script lang="ts">
  import { Icon } from '@immich/ui';
  import { mdiCheck } from '@mdi/js';
  import { SETUP_STEPS, type SetupStep } from '#lib/setup/setupSteps.ts';

  let { current }: { current: SetupStep } = $props();

  const currentIndex = $derived(SETUP_STEPS.findIndex((step) => step.id === current));
</script>

<ol aria-label="Setup steps" class="flex items-center gap-2 text-xs touch:items-start touch:gap-1">
  {#each SETUP_STEPS as step, index (step.id)}
    {@const done = index < currentIndex}
    {@const active = index === currentIndex}
    <li
      aria-current={active ? 'step' : undefined}
      class="flex items-center gap-2 touch:flex-1 touch:flex-col touch:items-stretch touch:gap-1.5 {index >
      0
        ? 'flex-1'
        : ''}"
    >
      {#if index > 0}
        <span aria-hidden="true" class="h-px flex-1 bg-hairline touch:hidden"></span>
      {/if}
      <span
        aria-hidden="true"
        class="hidden h-1 rounded-full touch:block {done || active ? 'bg-primary' : 'bg-light-200'}"
      ></span>
      <span class="flex items-center gap-1.5">
        <span
          aria-hidden="true"
          class="grid size-4.5 shrink-0 place-items-center rounded-full text-micro font-semibold touch:hidden {active
            ? 'bg-primary text-light'
            : 'bg-light-200 text-dark/80'}"
        >
          {#if done}<Icon icon={mdiCheck} size="12px" />{:else}{index + 1}{/if}
        </span>
        <span class={active ? 'font-medium text-light-950' : 'text-dark/80'}>
          {step.label}{#if done}<span class="sr-only"> (done)</span>{/if}
        </span>
      </span>
    </li>
  {/each}
</ol>
