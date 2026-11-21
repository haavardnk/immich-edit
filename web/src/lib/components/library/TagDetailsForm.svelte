<script lang="ts">
  import { userTags } from '#lib/browse/managedTags.ts';
  import { cssTagColor } from '#lib/browse/tagColor.ts';
  import {
    createdTagPath,
    siblingTags,
    tagDetailsPatch,
    tagNameProblem
  } from '#lib/browse/tagDetails.ts';
  import { splitTagValue, tagPathLabel } from '#lib/browse/tagTree.ts';
  import FormDialog from '#lib/components/FormDialog.svelte';
  import SearchableSelect from '#lib/components/SearchableSelect.svelte';
  import TextInput from '#lib/components/TextInput.svelte';
  import TagColorPicker from '#lib/components/library/TagColorPicker.svelte';
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { tagActions, type TagDetailsRequest } from '#lib/stores/tagActions.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { untrack } from 'svelte';

  let { request }: { request: TagDetailsRequest } = $props();

  const HINT_ID = 'tag-details-hint';

  const initial = untrack(() => request);
  const target = initial.kind === 'edit' ? initial.tag : null;
  const subtags = initial.kind === 'edit' ? initial.subtags : 0;
  const focus = initial.kind === 'edit' ? initial.focus : 'name';

  let name = $state(target?.name ?? '');
  let color = $state<string | null>(target ? cssTagColor(target.color) : null);
  let inside = $state<string[]>(
    initial.kind === 'create' && initial.parentId ? [initial.parentId] : []
  );
  let nameInput = $state<HTMLInputElement | null>(null);
  let colorGroup = $state<HTMLElement | null>(null);

  const renamable = $derived(!target || immichCapabilities.tagRename);
  const tags = $derived(userTags(library.tags));
  const parentId = $derived(inside[0] ?? null);
  const parentValue = $derived(
    target
      ? splitTagValue(target.value).parent
      : (tags.find((tag) => tag.id === parentId)?.value ?? '')
  );
  const problem = $derived(
    renamable
      ? tagNameProblem(
          name,
          siblingTags(library.tags, parentValue),
          parentValue,
          target?.id ?? null
        )
      : null
  );
  const valid = $derived(name.trim() !== '' && problem === null);
  const changed = $derived(
    !target || tagDetailsPatch(target, name, color, immichCapabilities.tagRename) !== null
  );
  const places = $derived(
    tags
      .map((tag) => ({ id: tag.id, label: tagPathLabel(tag.value) }))
      .sort((a, b) => a.label.localeCompare(b.label))
  );
  const hint = $derived.by(() => {
    if (problem) return problem;
    if (!renamable) return 'Renaming tags needs Immich 3.3 or later';
    if (subtags > 0) {
      return `Its ${countOf(subtags, 'sub-tag')} ${subtags === 1 ? 'moves' : 'move'} with it`;
    }
    return '';
  });

  $effect(() => {
    if (focus === 'color') colorGroup?.querySelector<HTMLElement>('[aria-checked="true"]')?.focus();
    else nameInput?.focus();
  });
</script>

<FormDialog
  title={target ? 'Tag details' : 'Create tag'}
  subtitle={target ? tagPathLabel(target.value) : undefined}
  submitLabel={target ? 'Save' : 'Create'}
  saving={tagActions.saving}
  disabled={!valid || !changed}
  onSubmit={() => void tagActions.save({ name, color, parentId: target ? null : parentId })}
  onClose={tagActions.closeDetails}
>
  {#snippet lead()}
    {#if !target && name.trim() !== '' && !problem}
      <p class="me-auto min-w-0 truncate text-caption text-dark/65">
        Creates {createdTagPath(parentValue, name)}
      </p>
    {/if}
  {/snippet}
  <div class="flex flex-col gap-1">
    <TextInput
      label="Name"
      color="neutral"
      bind:ref={nameInput}
      bind:value={name}
      readonly={!renamable}
      autocomplete="off"
      aria-invalid={problem ? 'true' : undefined}
      aria-describedby={HINT_ID}
      class="touch:[&_input]:h-12 touch:[&_input]:text-body"
    />
    <p id={HINT_ID} class="flex justify-between gap-3 text-caption text-dark/65">
      <span class={problem ? 'text-danger' : ''}>{hint}</span>
      <span class="shrink-0">Shown in Immich too</span>
    </p>
  </div>
  {#if !target}
    <div class="flex flex-col gap-2">
      <span class="text-sm font-medium" aria-hidden="true">Inside</span>
      <SearchableSelect
        color="neutral"
        options={places}
        bind:selected={inside}
        getId={(place) => place.id}
        getLabel={(place) => place.label}
        multiple={false}
        placeholder="Top level"
        label="Inside"
      />
    </div>
  {/if}
  <div class="flex flex-col gap-2">
    <span class="text-sm font-medium" aria-hidden="true">Color</span>
    <TagColorPicker value={color} onChange={(next) => (color = next)} bind:ref={colorGroup} />
  </div>
</FormDialog>
