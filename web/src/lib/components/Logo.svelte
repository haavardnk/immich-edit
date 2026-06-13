<script lang="ts">
  interface Props {
    size?: number;
    class?: string;
  }

  let { size = 24, class: className = '' }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 512 512"
  width={size}
  height={size}
  class={className}
  role="img"
  aria-label="immich-edit"
>
  <rect x="64" y="80" width="80" height="352" rx="40" fill="#6366F1" />
  <rect x="180" y="110" width="260" height="72" rx="36" fill="#FF4D4D" />
  <rect x="180" y="220" width="260" height="72" rx="36" fill="#F59E0B" />
  <rect x="180" y="330" width="260" height="72" rx="36" fill="#22C55E" />
</svg>
