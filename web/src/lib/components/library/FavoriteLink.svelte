<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { mdiHeartOutline } from '@mdi/js';
</script>

<div class="p-1">
  <a href="/favorites" class="flex items-center gap-2.5 py-1.5 px-2.5 rounded-lg hover:bg-white/5 transition-colors">
    <Icon path={mdiHeartOutline} size={16} class="opacity-40" />
    <span class="text-[13px] leading-tight">Favorites</span>
  </a>
</div>
