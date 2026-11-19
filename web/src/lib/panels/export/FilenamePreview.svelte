<script lang="ts">
  import { Icon } from '@immich/ui';
  import { mdiImageOutline } from '@mdi/js';

  let { id, name }: { id?: string; name: string } = $props();
</script>

<p {id} class="flex items-center gap-2 rounded-lg bg-ghost px-2.5 py-1.5 text-xs leading-snug">
  <Icon icon={mdiImageOutline} size="1.15em" class="shrink-0 text-dark/65" aria-hidden="true" />
  <span class="min-w-0 font-mono break-all text-dark">{name}</span>
</p>
