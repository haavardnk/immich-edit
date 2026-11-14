import {
  N_MAX_COMPONENTS_PER_LAYER,
  N_MAX_MASK_LAYERS,
  N_MAX_TOTAL_COMPONENTS
} from '#lib/types/edits.ts';
import type { maskCapacity } from '#lib/types/masks.ts';

type Capacity = ReturnType<typeof maskCapacity>;

const TOTAL = `You have used all ${N_MAX_TOTAL_COMPONENTS} shapes across every mask. Delete one to add another.`;

export function layerLimitText(cap: Capacity): string | null {
  if (cap.layersFull)
    return `You have used all ${N_MAX_MASK_LAYERS} masks. Delete one to add another.`;
  return cap.totalFull ? TOTAL : null;
}

export function shapeLimitText(cap: Capacity): string | null {
  if (cap.componentsFull) {
    return `This mask is full at ${N_MAX_COMPONENTS_PER_LAYER} shapes. Delete one to add another.`;
  }
  return cap.totalFull ? TOTAL : null;
}
