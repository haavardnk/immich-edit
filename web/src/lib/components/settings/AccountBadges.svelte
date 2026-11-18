<script lang="ts">
  import { authKindLabel } from '#lib/auth/account.ts';
  import type { SessionUser } from '#lib/api/auth.ts';
  import { Badge } from '@immich/ui';

  let { user }: { user: SessionUser } = $props();
</script>

{#if user.is_admin}<Badge size="tiny" color="primary">admin</Badge>{/if}
<Badge size="tiny" color="secondary">{authKindLabel(user.auth_kind)}</Badge>
