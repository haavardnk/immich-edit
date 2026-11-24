<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';

  const session = trackSession;
</script>

{#if session.located > 0}
  <CheckboxRow
    checked={session.replace}
    label="Also move {countOf(session.located, 'photo')} that already {session.located === 1
      ? 'has'
      : 'have'} a location"
    onChange={(next) => (session.replace = next)}
  />
{/if}
