---
title: OAuth sign-in
description: Reuse Immich's OAuth provider by registering immich-edit redirect URIs.
---

immich-edit delegates OAuth to Immich and never stores client IDs or secrets. Configure OAuth in
Immich first, then add immich-edit's redirect URIs at your identity provider.

## Setup

1. Enable and verify OAuth in Immich (**Administration** > **Settings**).
2. In your identity provider's Immich client, add `/login` and `/setup` for every origin users open
   immich-edit at (keep Immich's existing redirect URIs):

   ```yaml
   redirect_uris:
     - https://immich.example.com/auth/login
     - https://immich.example.com/user-settings
     - app.immich:///oauth-callback
     - https://edit.example.com/login
     - https://edit.example.com/setup
   ```

3. Open immich-edit's login page. The OAuth button appears within a minute.

## Which addresses to register

Match scheme, host, and port exactly (no trailing slash):

| Browser address | Register |
| --- | --- |
| `https://edit.example.com` | `https://edit.example.com/login` and `/setup` |
| `http://192.168.1.20:3000` | `http://192.168.1.20:3000/login` and `/setup` |
| `http://localhost:3000` | `http://localhost:3000/login` and `/setup` |
| `https://host.tailnet.ts.net` | `https://host.tailnet.ts.net/login` and `/setup` |

## Carried-over Immich settings

| Immich setting | Behavior in immich-edit |
| --- | --- |
| **Button Text** | Label on the login button |
| **Auto Launch** | Redirects immediately (append `?autoLaunch=0` or `?password=1` to bypass) |
| **Auto Register** | Provisions a new Immich and immich-edit user on first sign-in |
| **Role Claim** | Syncs the Immich administrator flag on every sign-in |

When Immich disables password login, **Use an Immich API key instead** remains available (also
required for rebinding an OAuth-only admin instance). Signing out revokes the underlying Immich
session.
