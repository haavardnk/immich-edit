<script lang="ts">
  import SectionList from '#lib/components/editor/sections/SectionList.svelte';
  import type { SectionGroup } from '#lib/components/editor/sections/sectionList.ts';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
  import { orderedTouchViews, touchSections } from '#lib/stores/developSections.svelte.ts';
  import { touchUi } from '#lib/stores/touchUi.svelte.ts';
  import { Button } from '@immich/ui';

  let { open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void } = $props();

  const groups = $derived<SectionGroup[]>(
    TOUCH_GROUPS.map((group) => ({
      id: group.id,
      label: group.label,
      rows: orderedTouchViews(group).map(({ view, key, hidden }) => ({
        id: key,
        label: view.label,
        hidden
      }))
    }))
  );

  function leaveHiddenViews(): void {
    for (const group of TOUCH_GROUPS) {
      const views = orderedTouchViews(group);
      const current = touchUi.viewOf(group.id);
      if (!views.find((entry) => entry.view.id === current)?.hidden) continue;
      const next = views.find((entry) => !entry.hidden);
      if (next) touchUi.setGroupView(group.id, next.view.id);
    }
  }

  function toggle(id: string): void {
    void touchSections.toggle(id);
    leaveHiddenViews();
  }

  function reset(): void {
    void touchSections.reset();
  }
</script>

<BottomSheet
  {open}
  {onOpenChange}
  title="Develop sections"
  close="done"
  detents={[{ fraction: 0.9 }]}
  bodyClass="px-2 pb-3"
>
  {#snippet header()}
    <Button
      size="small"
      shape="round"
      variant="ghost"
      color="secondary"
      class="bg-light-200 hover:bg-light-300"
      disabled={!touchSections.customised}
      onclick={reset}
    >
      Reset to default
    </Button>
  {/snippet}
  <SectionList
    {groups}
    variant="sheet"
    onreorder={(ids) => void touchSections.reorder(ids)}
    ontoggle={toggle}
  />
</BottomSheet>
