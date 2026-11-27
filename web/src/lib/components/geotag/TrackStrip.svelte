<script lang="ts">
  import { thumbUrl } from '#lib/api/assets.ts';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import { editorHref } from '#lib/editor/navigation.ts';
  import { openTrackPhoto } from '#lib/geotag/trackLoupe.ts';
  import { matchStatus } from '#lib/geotag/trackRows.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { currentPath } from '#lib/utils/route.ts';

  const session = trackSession;

  const DOTS = {
    ok: 'bg-primary',
    out: 'bg-warning-500',
    kept: 'bg-success'
  } as const;

  let strip = $state<HTMLElement | null>(null);

  $effect(() => {
    const id = session.focused;
    if (!id || !strip) return;
    strip
      .querySelector(`[data-strip-photo="${CSS.escape(id)}"]`)
      ?.scrollIntoView({ block: 'nearest', inline: 'center' });
  });

  function tap(event: MouseEvent, id: string): void {
    event.preventDefault();
    if (session.focused === id) openTrackPhoto(id);
    else session.hoverPin(id);
  }
</script>

<ul
  bind:this={strip}
  class="flex shrink-0 gap-1.5 overflow-x-auto px-4 pb-3 scrollbar-hidden"
  aria-label="Photos on the track"
>
  {#each session.rows as { asset, match } (asset.id)}
    {@const state = matchStatus(match)}
    {@const chosen = session.focused === asset.id}
    {@const named = [asset.originalFileName, state.text].filter(Boolean)}
    <li class="shrink-0" data-strip-photo={asset.id}>
      <TooltipLink
        tooltip={named.join(' · ')}
        href={editorHref(asset.id, currentPath())}
        onclick={(event) => tap(event, asset.id)}
        aria-label={named.join(', ')}
        aria-current={chosen ? 'true' : undefined}
        class="relative block size-16 overflow-hidden rounded-md outline-none {chosen
          ? 'ring-3 ring-primary'
          : ''}"
      >
        <img
          src={thumbUrl(asset.id, 'thumbnail')}
          alt=""
          loading="lazy"
          class="size-full object-cover"
        />
        <span
          class="absolute end-1 bottom-1 size-2.5 rounded-full ring-2 ring-light {DOTS[state.tone]}"
          aria-hidden="true"
        ></span>
      </TooltipLink>
    </li>
  {/each}
</ul>
