import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { monthCounts } from '#lib/stores/monthCounts.svelte.ts';
import { DayCounts } from './dayCounts.svelte';
import { localDays } from './scrubDays';

const searchCounts = vi.hoisted(() => vi.fn());

vi.mock('#lib/api/search.ts', () => ({ searchCounts, searchMonths: vi.fn() }));

type Answer = (counts: number[]) => void;

function held(): Answer[] {
  const answers: Answer[] = [];
  searchCounts.mockImplementation(() => new Promise<number[]>((resolve) => answers.push(resolve)));
  return answers;
}

function firstDay(month: string): number[] {
  return (localDays(month)?.keys ?? []).map((_, at) => (at === 0 ? 2 : 0));
}

beforeEach(() => {
  monthCounts.months = [
    { month: '2026-10', count: 2 },
    { month: '2026-09', count: 2 },
    { month: '2026-08', count: 2 }
  ];
});

afterEach(() => {
  searchCounts.mockReset();
});

describe('DayCounts', () => {
  it('loads one month at a time and keeps only the latest wanted month queued', async () => {
    const answers = held();
    const days = new DayCounts(() => ({ favorite: true }));
    days.want('2026-10');
    days.want('2026-09');
    days.want('2026-08');
    days.want('2026-10');
    expect(searchCounts).toHaveBeenCalledOnce();
    expect(searchCounts).toHaveBeenCalledWith({ favorite: true }, localDays('2026-10')?.edges);
    answers[0]?.(firstDay('2026-10'));
    await vi.waitFor(() => expect(searchCounts).toHaveBeenCalledTimes(2));
    expect(searchCounts).toHaveBeenLastCalledWith({ favorite: true }, localDays('2026-08')?.edges);
    expect(days.get('2026-10', 'desc')).toEqual([{ key: '2026-10-01', count: 2 }]);
    expect(days.get('2026-09', 'desc')).toBeNull();
  });

  it('does not retry a failed month until the month counts change', async () => {
    searchCounts.mockRejectedValue(new Error('down'));
    const days = new DayCounts(() => ({}));
    days.want('2026-10');
    await vi.waitFor(() => expect(searchCounts).toHaveBeenCalledOnce());
    await Promise.resolve();
    days.want('2026-10');
    expect(searchCounts).toHaveBeenCalledOnce();
    monthCounts.months = [...monthCounts.months];
    searchCounts.mockResolvedValue(firstDay('2026-10'));
    days.want('2026-10');
    await vi.waitFor(() => expect(days.get('2026-10', 'asc')).not.toBeNull());
    expect(searchCounts).toHaveBeenCalledTimes(2);
  });

  it('reloads a month whose answer arrives after the month counts changed', async () => {
    const answers = held();
    const days = new DayCounts(() => ({}));
    days.want('2026-10');
    monthCounts.months = [...monthCounts.months];
    days.want('2026-10');
    answers[0]?.(firstDay('2026-10'));
    await vi.waitFor(() => expect(searchCounts).toHaveBeenCalledTimes(2));
    expect(days.get('2026-10', 'desc')).toBeNull();
    answers[1]?.(firstDay('2026-10'));
    await vi.waitFor(() => expect(days.get('2026-10', 'desc')).not.toBeNull());
  });
});
