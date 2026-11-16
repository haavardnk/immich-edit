<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import { settingsHref } from '#lib/settings/pages.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { Link } from '@immich/ui';

  let { message, askAdmin }: { message: string; askAdmin: string } = $props();
</script>

<Notice color="warning" message={session.isAdmin ? message : `${message} ${askAdmin}`}>
  {#if session.isAdmin}
    <Link href={settingsHref('administration', 'models')} class="flex-none">Models settings</Link>
  {/if}
</Notice>
