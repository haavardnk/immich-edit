class UiStore {
  leftCollapsed = $state(false);
  rightCollapsed = $state(false);

  toggleLeft = (): void => {
    this.leftCollapsed = !this.leftCollapsed;
  };

  toggleRight = (): void => {
    this.rightCollapsed = !this.rightCollapsed;
  };
}

export const ui = new UiStore();
