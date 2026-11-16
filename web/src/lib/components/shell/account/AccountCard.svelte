<script lang="ts">
  import ImmichServerLine from '#lib/components/shell/account/ImmichServerLine.svelte';
  import type { SessionUser } from '#lib/api/auth.ts';
  import { Avatar, Badge } from '@immich/ui';

  let { user, inset = true }: { user: SessionUser; inset?: boolean } = $props();

  const name = $derived(user.name || user.email);
</script>

<div class="flex flex-col gap-2">
  <div
    class="flex items-center gap-3 touch:gap-3.5 {inset
      ? 'touch:rounded-2xl touch:bg-light-100 touch:p-3.5'
      : ''}"
  >
    <Avatar size="large" class="size-10 touch:size-13" {name} />
    <div class="min-w-0 flex-1">
      <div class="truncate text-sm font-semibold text-light-950 touch:text-base">{name}</div>
      <div class="truncate text-xs text-dark/65 touch:text-[13px]">{user.email}</div>
      {#if user.is_admin}
        <Badge size="tiny" color="primary" class="mt-1.5">admin</Badge>
      {/if}
    </div>
  </div>
  <ImmichServerLine
    class="truncate text-xs text-dark/65 touch:text-[12.5px] {inset ? 'touch:px-3.5' : ''}"
    dot
  />
</div>
