import { prefersReducedMotion } from 'svelte/motion';

export function flashElement(element: HTMLElement): void {
  element.scrollIntoView({
    block: 'center',
    behavior: prefersReducedMotion.current ? 'instant' : 'smooth'
  });
  element.removeAttribute('data-flash');
  void element.offsetWidth;
  element.setAttribute('data-flash', '');
  element.addEventListener('animationend', () => element.removeAttribute('data-flash'), {
    once: true
  });
}
