class GridViewportStore {
  topId = $state<string | null>(null);

  setTop(id: string | null): void {
    if (id !== this.topId) this.topId = id;
  }
}

export const gridViewport = new GridViewportStore();
