<script lang="ts">
  import { personThumbUrl, type PersonSummary } from '#lib/api/people.ts';
  import TooltipLink from '#lib/components/TooltipLink.svelte';

  let { person }: { person: PersonSummary } = $props();

  const name = $derived(person.name || 'Unknown');
</script>

<TooltipLink
  tooltip={name}
  href={`/people/${person.id}`}
  class="group flex w-16 shrink-0 flex-col items-center gap-1.5 rounded-lg text-center outline-none focus-visible:ring-2 focus-visible:ring-primary"
>
  <span
    class="no-callout size-16 overflow-hidden rounded-full bg-light-200 ring-1 ring-hairline group-hover:ring-2 group-hover:ring-primary/40"
  >
    <img src={personThumbUrl(person.id)} alt="" loading="lazy" class="size-full object-cover" />
  </span>
  <span data-truncate class="w-full truncate text-xs text-light-950">{name}</span>
</TooltipLink>
