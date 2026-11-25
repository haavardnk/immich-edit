import AxeBuilder from '@axe-core/playwright';
import type { Page } from '@playwright/test';
import { settleAnimations } from './animations';

type AxeResults = Awaited<ReturnType<AxeBuilder['analyze']>>;

export async function settledAxe(page: Page): Promise<AxeResults> {
  await settleAnimations(page);
  return new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
}

export async function seriousViolations(page: Page): Promise<string[]> {
  const results = await settledAxe(page);
  return results.violations
    .filter((violation) => violation.impact === 'serious' || violation.impact === 'critical')
    .map((violation) => violation.id);
}
