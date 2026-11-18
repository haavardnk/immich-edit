import { MediaQuery } from 'svelte/reactivity';

const narrowViewport = new MediaQuery('max-width: 1023px');

type Choice = { narrow: boolean; open: boolean };

class NavSidebar {
  #choice = $state<Choice | null>(null);

  get narrow(): boolean {
    return narrowViewport.current;
  }

  get open(): boolean {
    const choice = this.#choice;
    const narrow = this.narrow;
    return choice && choice.narrow === narrow ? choice.open : !narrow;
  }

  set open(open: boolean) {
    this.#choice = { narrow: this.narrow, open };
  }

  toggle = (): void => {
    this.open = !this.open;
  };
}

export const navSidebar = new NavSidebar();
