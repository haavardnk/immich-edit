<script lang="ts">
  import ContextMenuItem from '#lib/components/ContextMenuItem.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { paramDefault } from '#lib/editor/params/define.ts';
  import type { ParamDef } from '#lib/editor/params/types.ts';
  import { settingOf } from '#lib/editor/settingsSearch.ts';
  import { targetAvailable, targetToolOfSection } from '#lib/editor/targetTools.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import { targetAdjust } from '#lib/stores/targetAdjust.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import {
    mdiContentCopy,
    mdiCrosshairsGps,
    mdiPinOffOutline,
    mdiPinOutline,
    mdiRestore
  } from '@mdi/js';
  import { commitParamReset } from './paramReset';

  let { param }: { param: ParamDef } = $props();

  const touch = $derived(layout.mode === 'touch');
  const pinned = $derived(preferences.isPinned(param.id));
  const value = $derived(param.get(editor.edits, editor.isRaw));
  const modified = $derived(value !== paramDefault(param, editor.isRaw));
  const section = $derived(settingOf(param.id)?.section.id);
  const tool = $derived(section ? targetToolOfSection(section) : null);

  async function copyValue(): Promise<void> {
    try {
      await navigator.clipboard.writeText(param.format(Object.is(value, -0) ? 0 : value));
      toasts.push('success', `Copied ${param.label}`, { ttlMs: 2500 });
    } catch (error) {
      toasts.fail('Could not copy the value', error, 6000);
    }
  }
</script>

<ContextMenuItem
  icon={pinned ? mdiPinOffOutline : mdiPinOutline}
  onSelect={() => void preferences.togglePin(param.id)}
>
  {pinned ? 'Unpin' : 'Pin to Quick'}
</ContextMenuItem>
<ContextMenuItem
  icon={mdiRestore}
  detail={touch ? undefined : 'dbl-click'}
  disabled={!modified}
  onSelect={() => commitParamReset([param], `Reset ${param.history}`)}
>
  Reset {param.label}
</ContextMenuItem>
{#if !touch}
  <ContextMenuItem icon={mdiContentCopy} onSelect={() => void copyValue()}
    >Copy value</ContextMenuItem
  >
  {#if tool}
    {@const armed = tool}
    <ContextMenuItem
      icon={mdiCrosshairsGps}
      detail={keysFor('targetAdjust')}
      disabled={!targetAvailable(armed, editor.edits)}
      onSelect={() => targetAdjust.arm(armed)}
    >
      Adjust on photo
    </ContextMenuItem>
  {/if}
{/if}
