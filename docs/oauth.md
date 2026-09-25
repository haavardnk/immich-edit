---
layout: default
title: OAuth sign-in
parent: Run the server
nav_order: 6
permalink: /oauth/
---

# OAuth sign-in

immich-edit signs people in with the OAuth setup your Immich server already has. It has no OAuth
settings of its own, and it never sees the client ID or secret. The one change you make is at the
identity provider.

## How it works

1. A user presses the OAuth button on the immich-edit login page.
1. immich-edit asks Immich to start an OAuth sign-in and tells it to send the browser back to
   immich-edit's own `/login` page.
1. Immich sends the browser to the identity provider, which checks that return address against the
   client application's list of allowed redirect URIs.
1. After sign-in, the provider sends the browser back to immich-edit, which hands the result to
   Immich to finish. Immich answers with the user, and immich-edit signs them in.

Step 3 is why the provider has to know immich-edit's address. Immich keeps no list of its own, so
nothing changes in Immich.

## Set it up

1. Set up OAuth in Immich under **Administration** > **Settings** with
   [Immich's OAuth guide](https://docs.immich.app/administration/oauth), and check that Immich's own
   web login works with it.
1. At the identity provider, open the client application Immich uses and add two redirect URIs for
   each address people open immich-edit at:

   ```text
   http://192.168.1.20:3000/login
   http://192.168.1.20:3000/setup
   ```

   `/setup` is only used to claim a new instance with OAuth, but register it anyway.
1. Open the immich-edit login page. The button appears within a minute of enabling OAuth in Immich.

Keep the redirect URIs Immich already has. For example, an Authelia client used by Immich at
`immich.example.com` and immich-edit at `edit.example.com` ends up with:

```yaml
redirect_uris:
  - https://immich.example.com/auth/login
  - https://immich.example.com/user-settings
  - app.immich:///oauth-callback
  - https://edit.example.com/login
  - https://edit.example.com/setup
```

## Which addresses to register

The redirect URI must match the address in the browser exactly: scheme, host and port. A trailing
slash or a different port is enough for the provider to refuse it. Register every address in use:

| You open immich-edit at | Register |
| --- | --- |
| A LAN address, `http://192.168.1.20:3000` | `http://192.168.1.20:3000/login` and `/setup` |
| A hostname behind a reverse proxy, `https://edit.example.com` | `https://edit.example.com/login` and `/setup` |
| An SSH tunnel, `http://localhost:3000` | `http://localhost:3000/login` and `/setup` |
| Tailscale Serve, `https://host.tailnet-name.ts.net` | `https://host.tailnet-name.ts.net/login` and `/setup` |

Several providers only accept `https://` redirect URIs, apart from `localhost`. Serve immich-edit
through a [reverse proxy](deploy.md#reverse-proxy) with TLS for those. The ways to get
[browser previews on a local network](rendering.md#browser-previews-on-a-local-network) each change
the address you open, so register the new one when you switch.

## Settings that carry over from Immich

| Immich setting | Effect in immich-edit |
| --- | --- |
| **Button Text** | The label on the OAuth button |
| **Auto Launch** | The login page goes straight to the provider. Add `?autoLaunch=0` or `?password=1` to the login URL to stay on the page |
| **Auto Register** | A provider account with no Immich user gets one at first sign-in, and then an immich-edit account |
| **Role Claim** | Sets the Immich administrator flag when Immich creates the user, and with it immich-edit administrator rights |

Scope, claims and the signing algorithm act inside Immich as usual. immich-edit reads the Immich
user it gets back and copies the administrator flag at every sign-in.

## Accounts without a password

When Immich has password login turned off, the email and password form is hidden. **Use an Immich
API key instead** stays available, so an account without a working OAuth sign-in can still get in.
Create the key in Immich under **Account Settings** > **API Keys**.

Rebinding immich-edit to another Immich server takes an email and password or an API key, not
OAuth. An OAuth-only administrator creates an API key for it.

Signing out of an OAuth session also revokes the Immich session behind it.

## When it fails

The login page shows the provider's or Immich's error. The common ones are covered in
[troubleshooting](troubleshooting.md#setup-or-sign-in-fails):

- [The identity provider reports a redirect URI mismatch](troubleshooting.md#the-identity-provider-reports-a-redirect-uri-mismatch)
- [`Single sign-on was declined`](troubleshooting.md#single-sign-on-was-declined)
- [`The sign-in link expired. Start again.`](troubleshooting.md#the-sign-in-link-expired-start-again)
- [The OAuth button is missing](troubleshooting.md#the-oauth-button-is-missing)
