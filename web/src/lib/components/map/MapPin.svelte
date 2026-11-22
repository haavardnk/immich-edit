<script lang="ts">
  let { class: className = 'size-10' }: { class?: string } = $props();
</script>

<svg viewBox="0 0 24 24" class="drop-shadow-md {className}" aria-hidden="true">
  <path
    d="M20 10c0 5-5.5 10.2-7.4 11.8a1 1 0 0 1-1.2 0C9.5 20.2 4 15 4 10a8 8 0 0 1 16 0"
    class="fill-primary stroke-light"
    stroke-width="1.5"
  />
  <circle cx="12" cy="10" r="3" class="fill-light" />
</svg>
