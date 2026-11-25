<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';

  const session = trackSession;

  const count = $derived(session.timeShift.write.length);
</script>

{#if count > 0}
  <CheckboxRow
    checked={session.fixTime}
    label="Correct the capture time of {countOf(count, 'photo')}"
    class="h-auto items-start py-0.5 touch:h-auto touch:min-h-11"
    onChange={(next) => (session.fixTime = next)}
  >
    <span class="block text-dark/65">{session.fixCaption}</span>
  </CheckboxRow>
{/if}
