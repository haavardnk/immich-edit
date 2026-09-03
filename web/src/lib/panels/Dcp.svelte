<script lang="ts">
  import DcpPicker from './dcp/DcpPicker.svelte';
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import DeleteConfirmation from '$lib/components/DeleteConfirmation.svelte';
  import Notice from '$lib/components/Notice.svelte';
  import {
    segmentedControlClass,
    segmentedRadioItemClass
  } from '$lib/components/editor/controls/segmentedControl';
  import { mdiChevronDown, mdiClose, mdiTuneVariant, mdiUpload } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { session } from '$lib/stores/session.svelte';
  import { listDcps, matchDcp, importDcp, deleteDcp, type DcpMeta } from '$lib/api/dcp';
  import { ApiError } from '$lib/api/client';
  import { toasts } from '$lib/stores/toasts.svelte';
  import type { DcpIlluminant, DcpMode } from '$lib/types/edits';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { Collapsible, RadioGroup } from 'bits-ui';

  let dcps = $state<DcpMeta[]>([]);
  let loaded = $state(false);
  let importing = $state(false);
  let optionsOpen = $state(false);
  let pendingDelete = $state(false);
  let autoMatch = $state<DcpMeta | null>(null);
  let fileInput: HTMLInputElement | null = $state(null);

  const dcp = $derived(editor.edits.color.dcp);
  const selectedId = $derived(dcp.mode === 'profile' ? dcp.profile_id : null);
  const selected = $derived(dcps.find((profile) => profile.id === selectedId) ?? null);
  const missingSelected = $derived(loaded && !!selectedId && !selected);
  const cameraModel = $derived(editor.asset?.exifInfo?.model ?? null);
  const cameraMake = $derived(editor.asset?.exifInfo?.make ?? null);

  const illuminants: { value: DcpIlluminant; label: string }[] = [
    { value: 'interpolated', label: 'Auto' },
    { value: 'first', label: 'Illum 1' },
    { value: 'second', label: 'Illum 2' }
  ];

  $effect(() => {
    if (!loaded) void load();
  });

  $effect(() => {
    const model = cameraModel;
    const make = cameraMake;
    if (!model) {
      autoMatch = null;
    } else {
      void loadAutoMatch(model, make);
    }
  });

  async function load(): Promise<void> {
    try {
      dcps = await listDcps();
    } catch {
      dcps = [];
    }
    loaded = true;
  }

  async function loadAutoMatch(model: string, make: string | null): Promise<void> {
    try {
      autoMatch = await matchDcp(model, make);
    } catch {
      autoMatch = null;
    }
  }

  function select(mode: DcpMode, id: string | null): void {
    dcp.mode = mode;
    dcp.profile_id = id;
    pendingDelete = false;
    const action =
      mode === 'auto'
        ? 'DCP Auto Match'
        : mode === 'off'
          ? 'Default Color'
          : mode === 'flat'
            ? 'Flat Profile'
            : 'Select DCP Profile';
    void editor.onCommit(action);
  }

  function setIlluminant(value: DcpIlluminant): void {
    dcp.illuminant = value;
    void editor.onCommit('DCP Illuminant');
  }

  function toggle(
    field: 'use_tone_curve' | 'use_base_table' | 'use_look_table' | 'use_baseline_exposure',
    label: string,
    checked: boolean
  ): void {
    dcp[field] = checked;
    void editor.onCommit(label);
  }

  function triggerImport(): void {
    fileInput?.click();
  }

  async function onFile(e: Event): Promise<void> {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    const name = file.name.replace(/\.[^.]+$/, '');
    const bytes = new Uint8Array(await file.arrayBuffer());
    importing = true;
    try {
      const meta = await importDcp(name, bytes);
      await load();
      select('profile', meta.id);
    } catch (err) {
      if (err instanceof ApiError && err.status === 409) {
        const existing = err.message.split(':').pop()?.trim();
        await load();
        const duplicate = dcps.find((profile) => profile.id === existing);
        if (duplicate) {
          select(duplicate.bundled ? 'auto' : 'profile', duplicate.bundled ? null : duplicate.id);
          toasts.push(
            'info',
            duplicate.bundled
              ? 'Profile is already included in Auto match.'
              : 'Profile already imported — selected existing.'
          );
        }
      } else if (err instanceof ApiError && err.status === 400) {
        toasts.push('error', `Invalid profile: ${err.message}`);
      } else {
        toasts.push('error', 'Profile import failed.');
      }
    } finally {
      importing = false;
    }
  }

  async function confirmDelete(): Promise<void> {
    if (!selected || selected.bundled) return;
    try {
      await deleteDcp(selected.id);
      select('auto', null);
      await load();
    } catch {
      toasts.push('error', 'Profile delete failed.');
    }
  }
</script>

<div class="flex flex-col gap-1 pb-1">
  <input
    bind:this={fileInput}
    type="file"
    accept=".dcp"
    class="hidden"
    onchange={(e) => void onFile(e)}
  />

  <div class="flex items-center gap-1">
    <div class="min-w-0 flex-1">
      <DcpPicker
        profiles={dcps}
        mode={dcp.mode}
        {selectedId}
        {autoMatch}
        {cameraModel}
        onSelect={select}
      />
    </div>
    {#if selected && !selected.bundled && session.isAdmin}
      <DeleteConfirmation
        bind:pending={pendingDelete}
        label="Delete imported profile"
        confirmLabel="Confirm delete profile"
        onconfirm={confirmDelete}
      />
    {/if}
  </div>

  {#if session.isAdmin}
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      fullWidth
      class="panel-action h-7"
      leadingIcon={mdiUpload}
      disabled={importing}
      onclick={triggerImport}
    >
      {importing ? 'Importing…' : 'Import .dcp profile'}
    </Button>
  {/if}

  {#if missingSelected}
    <Notice color="warning" message="Referenced profile is unavailable" class="mx-1">
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiClose}
        title="Return to auto match"
        aria-label="Return to auto match"
        onclick={() => select('auto', null)}
      />
    </Notice>
  {/if}

  {#if dcp.mode !== 'off' && dcp.mode !== 'flat'}
    <Collapsible.Root bind:open={optionsOpen} class="flex flex-col gap-1">
      <Collapsible.Trigger
        class="flex h-7 items-center gap-1.5 rounded-md px-2 text-[11px] font-medium text-dark/65 transition-colors hover:bg-ghost hover:text-dark"
      >
        <Icon icon={mdiTuneVariant} size="13px" aria-hidden="true" />
        <span class="flex-1 text-left">Profile options</span>
        <Icon
          icon={mdiChevronDown}
          size="13px"
          class="transition-transform {optionsOpen ? 'rotate-180' : ''}"
          aria-hidden="true"
        />
      </Collapsible.Trigger>

      <Collapsible.Content>
        {#if optionsOpen}
          <div class="flex flex-col gap-1 border-t border-hairline pt-1">
            <div class="flex flex-col gap-1">
              <span class="text-[10px] font-medium text-dark/65">Illuminant</span>
              <RadioGroup.Root
                bind:value={() => dcp.illuminant, (v) => setIlluminant(v as DcpIlluminant)}
                orientation="horizontal"
                aria-label="Illuminant"
                class={segmentedControlClass}
              >
                {#each illuminants as illuminant (illuminant.value)}
                  <RadioGroup.Item
                    value={illuminant.value}
                    class="{segmentedRadioItemClass} flex-1"
                  >
                    {illuminant.label}
                  </RadioGroup.Item>
                {/each}
              </RadioGroup.Root>
            </div>

            <div class="flex flex-col gap-0.5">
              <CheckboxRow
                label="Base table (HueSatMap)"
                checked={dcp.use_base_table}
                onChange={(v) => toggle('use_base_table', 'DCP Base Table', v)}
              />
              <CheckboxRow
                label="Tone curve"
                checked={dcp.use_tone_curve}
                onChange={(v) => toggle('use_tone_curve', 'DCP Tone Curve', v)}
              />
              <CheckboxRow
                label="Look table"
                checked={dcp.use_look_table}
                onChange={(v) => toggle('use_look_table', 'DCP Look Table', v)}
              />
              <CheckboxRow
                label="Baseline exposure"
                checked={dcp.use_baseline_exposure}
                onChange={(v) => toggle('use_baseline_exposure', 'DCP Baseline Exposure', v)}
              />
            </div>
          </div>
        {/if}
      </Collapsible.Content>
    </Collapsible.Root>
  {/if}
</div>
