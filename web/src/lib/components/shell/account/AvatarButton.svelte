<script lang="ts">
  import AttentionDot from '#lib/components/jobs/AttentionDot.svelte';
  import { accountDrawer } from '#lib/components/shell/account/accountDrawer.svelte.ts';
  import { accountLabel } from '#lib/components/shell/account/accountLabel.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { Avatar, Button } from '@immich/ui';

  const name = $derived(session.user?.name || session.user?.email || '?');
  const label = $derived(accountLabel(name));
</script>

<div class="relative shrink-0">
  <Button
    variant="ghost"
    color="secondary"
    shape="round"
    class="size-11 p-0"
    title="Account"
    aria-label={jobs.attention ? `${label}, a job finished with failures` : label}
    aria-haspopup="dialog"
    aria-expanded={accountDrawer.open}
    onclick={accountDrawer.show}
  >
    <Avatar size="small" class="size-8 text-label font-bold" {name} />
  </Button>
  {#if jobs.attention}
    <AttentionDot class="absolute top-1 right-1 ring-2 ring-light" />
  {/if}
</div>
