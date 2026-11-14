<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';

  let {
    icon,
    label,
    detail,
    disabled = false,
    onclick
  }: {
    icon: string;
    label: string;
    detail: string;
    disabled?: boolean;
    onclick: () => void;
  } = $props();
</script>

<MenuRow {icon} {label} class="hidden touch:flex" aria-label={label} {disabled} {onclick}>
  <span class="shrink-0 text-[13px] text-dark/65">{detail}</span>
  <Icon icon={mdiChevronRight} size="20px" class="shrink-0 text-dark/65" aria-hidden="true" />
</MenuRow>
