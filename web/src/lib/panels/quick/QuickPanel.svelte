<script lang="ts">
  import Disclosure from '#lib/components/Disclosure.svelte';
  import { commitParamReset } from '#lib/components/editor/controls/paramReset.ts';
  import { paramsModified } from '#lib/editor/params/define.ts';
  import type { ParamDef } from '#lib/editor/params/types.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { quickSection } from '#lib/stores/quickSection.svelte.ts';
  import { mdiPinOutline } from '@mdi/js';
  import QuickRows from './QuickRows.svelte';

  let { params }: { params: readonly ParamDef[] } = $props();

  const modified = $derived(paramsModified(editor.edits, params, editor.isRaw));
</script>

<div data-panel="quick">
  <Disclosure
    open={quickSection.open}
    title="Quick"
    icon={mdiPinOutline}
    {modified}
    onReset={() => commitParamReset(params, 'Reset Quick')}
    onOpenChange={quickSection.setOpen}
  >
    <div class="bg-recess px-3 pt-1 pb-2">
      <QuickRows {params} />
    </div>
  </Disclosure>
</div>
