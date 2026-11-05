<script lang="ts">
  let { value }: { value: number } = $props();

  const RADIUS = 9;
  const CIRCUMFERENCE = 2 * Math.PI * RADIUS;
</script>

<svg viewBox="0 0 24 24" class="size-6 shrink-0 -rotate-90" aria-hidden="true">
  <circle cx="12" cy="12" r={RADIUS} fill="none" stroke-width="3" class="stroke-dark/15" />
  <circle
    cx="12"
    cy="12"
    r={RADIUS}
    fill="none"
    stroke-width="3"
    stroke-linecap="round"
    stroke-dasharray={`${Math.max(0.5, value * CIRCUMFERENCE)} ${CIRCUMFERENCE}`}
    class="stroke-primary motion-safe:transition-[stroke-dasharray] motion-safe:duration-300"
  />
</svg>
