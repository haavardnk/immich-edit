<script lang="ts">
  import type { PlanStats } from '#lib/api/diagnostics.ts';
  import { planLabel } from '#lib/diagnostics/format.ts';
  import { Heading, Text } from '@immich/ui';

  let { plans }: { plans: PlanStats[] } = $props();
</script>

<section class="space-y-2 pt-5">
  <Heading tag="h2" size="tiny" color="muted" fontWeight="medium">Render plans</Heading>
  {#if plans.length === 0}
    <Text size="tiny" color="muted">No renders recorded yet.</Text>
  {:else}
    <table class="w-full text-xs" aria-label="Render plans">
      <thead>
        <tr class="text-[10px] uppercase text-dark/65">
          <th class="py-1 text-left font-normal">Renderer</th>
          <th class="py-1 text-left font-normal">Plan</th>
          <th class="py-1 text-right font-normal">Renders</th>
          <th class="py-1 text-right font-normal">Last tiles</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-hairline font-mono tabular-nums">
        {#each plans as p (`${p.renderer}-${p.plan}-${p.edge}`)}
          <tr>
            <td class="py-1 font-sans text-dark/65">{p.renderer.toUpperCase()}</td>
            <td class="py-1 font-sans">{planLabel(p)}</td>
            <td class="py-1 text-right">{p.renders}</td>
            <td class="py-1 text-right">{p.last_tiles}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>
