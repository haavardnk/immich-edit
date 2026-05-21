export type AspectRatio = 'free' | 'original' | '1:1' | '4:3' | '3:2' | '16:9' | '5:4' | '7:5';

export const ASPECT_RATIOS: { id: AspectRatio; label: string; value: number | null }[] = [
  { id: 'free', label: 'Free', value: null },
  { id: 'original', label: 'Original', value: null },
  { id: '1:1', label: '1:1', value: 1 },
  { id: '4:3', label: '4:3', value: 4 / 3 },
  { id: '3:2', label: '3:2', value: 3 / 2 },
  { id: '16:9', label: '16:9', value: 16 / 9 },
  { id: '5:4', label: '5:4', value: 5 / 4 },
  { id: '7:5', label: '7:5', value: 7 / 5 },
];

class UiStore {
  leftCollapsed = $state(false);
  rightCollapsed = $state(false);
  filmstripCollapsed = $state(false);
  searchQuery = $state('');

  toggleLeft = (): void => {
    this.leftCollapsed = !this.leftCollapsed;
  };

  toggleRight = (): void => {
    this.rightCollapsed = !this.rightCollapsed;
  };

  toggleFilmstrip = (): void => {
    this.filmstripCollapsed = !this.filmstripCollapsed;
  };
}

export const ui = new UiStore();
