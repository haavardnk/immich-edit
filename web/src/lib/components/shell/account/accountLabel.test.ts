import { describe, expect, it } from 'vitest';
import { accountLabel, avatarInitials } from './accountLabel';

describe('accountLabel', () => {
  it('names the avatar with the initials it shows', () => {
    expect(avatarInitials('test user')).toBe('TU');
    expect(avatarInitials('Ada King Lovelace')).toBe('AK');
    expect(accountLabel('Test User')).toBe('Account TU');
    expect(accountLabel('hanne@example.com')).toBe('Account H');
  });

  it('falls back to a plain name when the avatar shows no letters', () => {
    expect(accountLabel('?')).toBe('Account');
    expect(accountLabel('')).toBe('Account');
  });
});
