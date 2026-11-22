export type TrayState = 'closed' | 'open' | 'carrying';

export class MapTrayState {
  state = $state<TrayState>('closed');
  detent = $state(0);

  open = (): void => {
    if (this.state === 'closed') this.detent = 0;
    this.state = 'open';
  };

  close = (): void => {
    this.state = 'closed';
  };

  carry = (): void => {
    if (this.state === 'open') this.state = 'carrying';
  };
}
