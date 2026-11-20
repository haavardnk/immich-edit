export function avatarInitials(name: string): string {
  return name
    .split(' ')
    .map((part) => part.at(0) ?? '')
    .join('')
    .slice(0, 2)
    .toUpperCase();
}

export function accountLabel(name: string): string {
  const initials = avatarInitials(name);
  return /[\p{L}\p{N}]/u.test(initials) ? `Account ${initials}` : 'Account';
}
