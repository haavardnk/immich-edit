export function listSummary(labels: string[]): string {
  const [first, ...rest] = labels;
  if (first === undefined) return '';
  return rest.length === 0 ? first : `${first} +${rest.length}`;
}
