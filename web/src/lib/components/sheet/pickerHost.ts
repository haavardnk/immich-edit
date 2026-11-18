import { getContext, setContext } from 'svelte';

export type PickerHost = { readonly inline: boolean };

const KEY = Symbol('pickerHost');

const DETACHED: PickerHost = { inline: false };

export function providePickerHost(host: PickerHost): void {
  setContext(KEY, host);
}

export function pickerHost(): PickerHost {
  return getContext<PickerHost | undefined>(KEY) ?? DETACHED;
}
