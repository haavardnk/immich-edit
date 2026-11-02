<script lang="ts">
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiWeatherNight, mdiWeatherSunny } from '@mdi/js';

  const toLight = $derived(appearance.scheme === 'dark');
  const label = $derived(toLight ? 'Switch to light theme' : 'Switch to dark theme');
</script>

<IconButton
  size="medium"
  variant="ghost"
  color="secondary"
  shape="rectangle"
  class="max-sm:hidden"
  icon={toLight ? mdiWeatherSunny : mdiWeatherNight}
  title={label}
  aria-label={label}
  onclick={() => appearance.toggleMode()}
/>
