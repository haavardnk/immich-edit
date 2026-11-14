<script lang="ts">
  import { countOf } from '#lib/utils/plural.ts';
  import { Icon } from '@immich/ui';
  import { mdiStar } from '@mdi/js';

  let { rating, class: className = '' }: { rating: number; class?: string } = $props();
</script>

<span
  role="img"
  aria-label={countOf(rating, 'star')}
  class="flex shrink-0 items-center gap-0.5 text-[10px] {className}"
>
  <Icon icon={mdiStar} size="11px" aria-hidden="true" />
  {rating}
</span>
