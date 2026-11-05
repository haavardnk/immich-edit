<script lang="ts">
  import { personThumbUrl, type PersonSummary } from '#lib/api/people.ts';

  let { person }: { person: PersonSummary } = $props();
</script>

<a
  href={`/people/${person.id}`}
  class="group flex w-20 shrink-0 flex-col items-center gap-1.5 rounded-lg p-1 text-center outline-none focus-visible:ring-2 focus-visible:ring-primary"
>
  <span
    class="no-callout size-16 overflow-hidden rounded-full bg-light-200 ring-primary/40 group-hover:ring-2"
  >
    <img src={personThumbUrl(person.id)} alt="" loading="lazy" class="size-full object-cover" />
  </span>
  <span class="w-full truncate text-xs text-light-950">{person.name || 'Unknown'}</span>
</a>
