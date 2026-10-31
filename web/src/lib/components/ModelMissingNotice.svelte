<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import { session } from '#lib/stores/session.svelte.ts';
  import { Link } from '@immich/ui';

  let { message, askAdmin }: { message: string; askAdmin: string } = $props();
</script>

<Notice color="warning" message={session.isAdmin ? message : `${message} ${askAdmin}`}>
  {#if session.isAdmin}
    <Link href="/settings#models" class="flex-none">Models settings</Link>
  {/if}
</Notice>
