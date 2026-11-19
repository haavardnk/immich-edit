import type { CullState } from '#lib/browse/cullState.ts';
import { LABEL_NAMES } from '#lib/stores/labels.ts';
import { countOf } from '#lib/utils/plural.ts';

export type CullKind = 'rating' | 'favorite' | 'reject' | 'label';

export function markText(kind: CullKind, state: CullState): string {
  switch (kind) {
    case 'rating':
      return state.rating > 0 ? countOf(state.rating, 'star') : 'No rating';
    case 'favorite':
      return state.isFavorite ? 'Favorite' : 'Not a favorite';
    case 'reject':
      return state.rejected ? 'Rejected' : 'Not rejected';
    case 'label':
      return state.label ? `${LABEL_NAMES[state.label]} label` : 'No label';
  }
}
