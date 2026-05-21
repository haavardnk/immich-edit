<script lang="ts">
  let {
    label,
    value = $bindable(),
    min,
    max,
    step = 0.1,
    onLive,
    onCommit
  }: {
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    onLive: () => void;
    onCommit: () => void;
  } = $props();
</script>

<div class="flex flex-col gap-1">
  <div class="flex items-center justify-between text-xs">
    <span class="opacity-80">{label}</span>
    <span class="font-mono opacity-60">{value.toFixed(2)}</span>
  </div>
  <input
    type="range"
    class="range range-sm range-primary"
    {min}
    {max}
    {step}
    bind:value
    oninput={onLive}
    onchange={onCommit}
  />
</div>
