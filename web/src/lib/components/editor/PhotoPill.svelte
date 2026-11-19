<script lang="ts">
  import { Icon } from '@immich/ui';

  let {
    lead,
    label,
    tail = '',
    mono = false,
    icon,
    hint,
    class: className = ''
  }: {
    lead: string;
    label: string;
    tail?: string;
    mono?: boolean;
    icon?: string;
    hint?: string;
    class?: string;
  } = $props();
</script>

<span
  role="status"
  data-photo-pill
  class="pointer-events-none absolute top-3 left-1/2 z-20 max-w-[calc(100%-1.5rem)] -translate-x-1/2 truncate rounded-full bg-black/70 px-3 py-1 text-xs whitespace-nowrap text-white shadow-lg backdrop-blur-sm {className}"
  >{#if icon}<Icon
      {icon}
      size="14px"
      class="me-1.5 inline-block align-[-3px]"
      aria-hidden="true"
    />{/if}{lead}
  <span class="font-semibold text-image-accent {mono ? 'font-mono' : ''}">{label}</span>{tail
    ? ` ${tail}`
    : ''}{#if hint}<span class="ms-2 text-white/70">{hint}</span>{/if}</span
>
