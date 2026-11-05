<script lang="ts">
  import AttentionDot from '#lib/components/jobs/AttentionDot.svelte';
  import { accountDrawer } from '#lib/components/shell/touch/accountDrawer.svelte.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { Avatar, Button } from '@immich/ui';

  const name = $derived(session.user?.name || session.user?.email || '?');
</script>

<div class="relative shrink-0">
  <Button
    variant="ghost"
    color="secondary"
    shape="round"
    class="size-11 p-0"
    aria-label={jobs.attention ? 'Account, a job finished with failures' : 'Account'}
    aria-haspopup="dialog"
    aria-expanded={accountDrawer.open}
    onclick={accountDrawer.show}
  >
    <Avatar size="small" {name} />
  </Button>
  {#if jobs.attention}
    <AttentionDot class="absolute top-1 right-1 ring-2 ring-light-50" />
  {/if}
</div>
