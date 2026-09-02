<script lang="ts">
  interface Props {
    size?: number;
    class?: string;
    decorative?: boolean;
  }

  let { size = 24, class: className = '', decorative = false }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 512 512"
  width={size}
  height={size}
  class={className}
  role={decorative ? undefined : 'img'}
  aria-label={decorative ? undefined : 'immich-edit'}
  aria-hidden={decorative ? 'true' : undefined}
>
  <path
    d="M64 182V80h102M346 80h102v102M448 330v102H346M166 432H64V330"
    fill="none"
    stroke="var(--color-logo-primary)"
    stroke-width="38"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
  <rect x="150" y="144" width="52" height="224" rx="26" fill="var(--immich-ui-dark)" />
  <path d="M262 144h116v42h-68v48h58v42h-58v50h68v42H262Z" fill="var(--immich-ui-dark)" />
</svg>
