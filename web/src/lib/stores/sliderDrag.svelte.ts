class SliderDragStore {
  setting = $state<string | null>(null);

  start = (setting: string | undefined): void => {
    this.setting = setting ?? '';
  };

  end = (): void => {
    this.setting = null;
  };
}

export const sliderDrag = new SliderDragStore();
