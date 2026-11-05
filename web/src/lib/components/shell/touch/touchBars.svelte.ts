class TouchBarsStore {
  hidden = $state(false);

  set = (hidden: boolean): void => {
    this.hidden = hidden;
  };
}

export const touchBars = new TouchBarsStore();
