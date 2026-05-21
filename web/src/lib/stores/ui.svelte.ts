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
