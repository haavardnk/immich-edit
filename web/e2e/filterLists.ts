import type { Locator } from '@playwright/test';

export function filterGroup(panel: Locator, name: string): Locator {
  return panel.getByRole('group', { name, exact: true });
}

export function filterRow(group: Locator, name: string): Locator {
  return group.getByRole('checkbox', { name, exact: true });
}

export function rowNames(group: Locator): Promise<string[]> {
  return group
    .getByRole('checkbox')
    .evaluateAll((boxes) => boxes.map((box) => box.getAttribute('aria-label') ?? ''));
}
