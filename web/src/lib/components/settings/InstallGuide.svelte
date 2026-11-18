<script lang="ts">
  import type { InstallGuide } from '#lib/pwa/installState.ts';

  let { guide }: { guide: InstallGuide } = $props();
</script>

{#snippet step(number: number)}
  <span
    class="grid size-6 flex-none place-items-center rounded-full bg-primary/15 text-xs font-semibold text-primary"
    aria-hidden="true">{number}</span
  >
{/snippet}

<div class="space-y-4 text-sm text-light-950">
  {#if guide === 'ios'}
    <p>Installing from this browser takes three steps:</p>
    <ol class="space-y-3">
      <li class="flex items-center gap-3">
        {@render step(1)}<span>Tap <strong>Share</strong> in the browser toolbar</span>
      </li>
      <li class="flex items-center gap-3">
        {@render step(2)}<span>Choose <strong>Add to Home Screen</strong></span>
      </li>
      <li class="flex items-center gap-3">
        {@render step(3)}<span
          >Keep <strong>Open as Web App</strong> on and tap <strong>Add</strong></span
        >
      </li>
    </ol>
  {:else if guide === 'safari-mac'}
    <p>Safari adds web apps to the Dock. It takes two steps:</p>
    <ol class="space-y-3">
      <li class="flex items-center gap-3">
        {@render step(1)}<span>Choose <strong>File › Add to Dock</strong> in the menu bar</span>
      </li>
      <li class="flex items-center gap-3">
        {@render step(2)}<span>Keep the name and click <strong>Add</strong></span>
      </li>
    </ol>
  {:else}
    <p>
      This browser has not offered a one-step install. If immich-edit isn't installed yet, most
      browsers can still add it:
    </p>
    <ol class="space-y-3">
      <li class="flex items-center gap-3">
        {@render step(1)}<span>Open the browser menu</span>
      </li>
      <li class="flex items-center gap-3">
        {@render step(2)}<span
          >Choose <strong>Install app</strong> or <strong>Add to Home screen</strong></span
        >
      </li>
    </ol>
  {/if}
  {#if guide !== 'browser'}
    <p class="text-xs text-dark/65">
      The installed app has its own sign-in. You'll log in once more inside it.
    </p>
  {/if}
</div>
