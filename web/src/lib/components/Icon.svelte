<script lang="ts">
  let {
    path,
    size = 24,
    class: className = '',
  }: { path: string; size?: number; class?: string } = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 24 24"
  width={size}
  height={size}
  fill="currentColor"
  class={className}
>
  <path d={path} />
</svg>
