<script lang="ts">
  import TrackFixTime from '#lib/components/geotag/TrackFixTime.svelte';
  import TrackMoveLocated from '#lib/components/geotag/TrackMoveLocated.svelte';
</script>

<div class="flex flex-col gap-2 empty:hidden">
  <TrackMoveLocated />
  <TrackFixTime />
</div>
