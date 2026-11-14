<script lang="ts">
  import DeleteConfirmation from '#lib/components/DeleteConfirmation.svelte';
  import NameField from '#lib/components/NameField.svelte';
  import { presence } from '#lib/components/sheet/presence.svelte.ts';
  import { deleteVersion, openVersionInLoupe, versionHref } from '#lib/editor/versionActions.ts';
  import type { Version } from '#lib/editor/versions.ts';
  import { clickGuard } from '#lib/gestures/clickGuard.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDotsHorizontal, mdiPencil } from '@mdi/js';
  import VersionActions from './VersionActions.svelte';
  import VersionThumb from './VersionThumb.svelte';
  import type { VersionAction } from './versionAction.ts';

  let {
    version,
    current,
    editing,
    summary,
    onrename,
    onendrename,
    onnavigate
  }: {
    version: Version;
    current: boolean;
    editing: boolean;
    summary: string;
    onrename: () => void;
    onendrename: () => void;
    onnavigate: () => void;
  } = $props();

  let deleting = $state(false);
  let actionsOpen = $state(false);

  const store = editor.versions;
  const copy = $derived(version.copyIndex !== null);
  const sheet = presence(() => actionsOpen);
  const guard = clickGuard();

  async function rename(name: string): Promise<void> {
    if (await store.rename(version.id, name)) onendrename();
  }

  async function remove(): Promise<void> {
    await deleteVersion(store, version.id, onnavigate);
  }

  function holdKey(event: KeyboardEvent): void {
    if (event.key !== ' ') return;
    event.preventDefault();
    if (!event.repeat) store.hold.start(version);
  }

  function releaseKey(event: KeyboardEvent): void {
    if (event.key === ' ') store.hold.end(version.id);
  }

  function act(action: VersionAction): void {
    if (action === 'rename') onrename();
    else if (action === 'loupe') void openVersionInLoupe(version.id);
    else void remove();
  }
</script>

<div
  class="group flex items-center transition-colors hover:bg-ghost touch:border-b touch:border-hairline touch:hover:bg-transparent {current
    ? 'bg-primary/10 touch:hover:bg-primary/10'
    : ''}"
  data-version-row
>
  {#if editing}
    <div
      class="flex h-8 min-w-0 flex-1 items-center gap-2.5 pr-1.5 pl-3 touch:h-auto touch:gap-3.5 touch:px-4 touch:py-2"
    >
      <VersionThumb {version} {current} />
      <NameField
        initial={version.named ? version.name : ''}
        ariaLabel="Version name"
        required={false}
        onsave={rename}
        oncancel={onendrename}
      />
    </div>
  {:else}
    <a
      href={versionHref(version.id)}
      aria-current={current ? 'page' : undefined}
      class="flex h-8 min-w-0 flex-1 items-center gap-2.5 pl-3 text-xs touch:h-auto touch:gap-3.5 touch:py-2 touch:pl-4 {current
        ? 'text-primary touch:text-dark'
        : ''}"
      onclick={onnavigate}
      onkeydown={holdKey}
      onkeyup={releaseKey}
      onblur={() => store.hold.end(version.id)}
      {@attach guard.attachment}
    >
      <VersionThumb {version} {current} onhold={guard.arm} />
      <span class="flex min-w-0 flex-1 flex-col touch:gap-0.5">
        <span class="flex min-w-0 items-center gap-1.5">
          <span class="truncate touch:text-base touch:font-medium">{version.name}</span>
          {#if current}
            <span
              class="hidden shrink-0 text-[11px] font-semibold tracking-wider text-primary uppercase touch:inline"
              >Current</span
            >
          {/if}
        </span>
        {#if summary}
          <span class="hidden truncate text-[13px] text-dark/65 touch:block">{summary}</span>
        {/if}
      </span>
    </a>
    {#if copy}
      <div
        class="flex items-center pr-1.5 transition-opacity touch:hidden {deleting
          ? ''
          : 'opacity-0 group-hover:opacity-100 group-focus-within:opacity-100'}"
      >
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          icon={mdiPencil}
          title="Rename"
          aria-label="Rename copy"
          onclick={onrename}
        />
        <DeleteConfirmation
          bind:pending={deleting}
          label="Delete copy"
          title="Delete"
          confirmLabel="Confirm delete copy"
          onconfirm={remove}
        />
      </div>
    {/if}
    <IconButton
      size="medium"
      variant="ghost"
      color="secondary"
      shape="round"
      class="mr-1 hidden touch:inline-flex"
      icon={mdiDotsHorizontal}
      title="Version actions"
      aria-label="Actions for {version.name}"
      onclick={() => (actionsOpen = true)}
    />
  {/if}
  {#if sheet.mounted}
    <VersionActions
      open={actionsOpen}
      name={version.name}
      {copy}
      onOpenChange={(next) => (actionsOpen = next)}
      onexited={sheet.exited}
      onaction={act}
    />
  {/if}
</div>
