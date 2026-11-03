<script lang="ts">
  import ParamSection from '#lib/components/editor/controls/ParamSection.svelte';
  import { PRESENCE, TONE } from '#lib/editor/params/basic.ts';
  import WhiteBalanceSection from './basic/WhiteBalanceSection.svelte';
</script>

<div class="flex flex-col divide-y divide-dark/10">
  <div class="flex flex-col gap-1 pb-1.5">
    <WhiteBalanceSection />
  </div>
  <div class="flex flex-col gap-1 py-1.5">
    <ParamSection section={TONE} />
  </div>
  <div class="flex flex-col gap-1 pt-1.5">
    <ParamSection section={PRESENCE} />
  </div>
</div>
