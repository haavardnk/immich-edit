<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';

  const session = trackSession;

  const count = $derived(session.captureWrites);
</script>

{#if count > 0}
  <CheckboxRow
    checked={session.fixTime}
    label="Save corrected capture time in Immich ({countOf(count, 'photo')})"
    onChange={(next) => (session.fixTime = next)}
  />
{/if}
