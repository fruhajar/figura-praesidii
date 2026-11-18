# Figura Praesidii: mobile

Slice 3: measure what the device actually emits. This replaces the device-ID lookup
idea, which has no lawful corpus to query (see [scope.md](scope.md)).

Do not ask a database what it knows about the handset. Measure the handset.

## What to measure

- Which advertising ID the device currently presents, and when it last changed.
- Which trackers and SDK endpoints each app contacts, and how often.
- What the device broadcasts over Wi-Fi and BLE while idle, and whether MAC
  randomisation is actually in effect.
- Which apps hold location, contacts, camera, microphone and clipboard access, and
  when each last used it.

Findings are actionable within a minute: reset the advertising ID, revoke a
permission, fix randomisation. Nothing leaves the device.

## Android is the target

`VpnService` gives a local-only VPN that observes outbound DNS and TLS SNI and
attributes connections per app, with no root and no traffic leaving the handset. This
is how TrackerControl, NetGuard and Blokada work, and it is the core of the product.

What Android will *not* allow, so plan around it:

| Wanted | Reality |
|---|---|
| IMEI | Unavailable since Android 10. Needs `READ_PRIVILEGED_PHONE_STATE`, system apps only. |
| Own Wi-Fi MAC | Returns `02:00:00:00:00:00` since Android 6. |
| Other apps' advertising IDs | Not accessible. Only the app's own. |
| Full installed-package list | Restricted from Android 11. `QUERY_ALL_PACKAGES` needs a Play Store justification. |
| Per-app permission state | **Available** via `PackageManager`, and genuinely useful. |

**Distribute through F-Droid first.** It avoids the Play policy fight over
`QUERY_ALL_PACKAGES` and VPN-based inspection, and it matches the project's own
argument that apps worth trusting should be obtainable outside the OS vendor's
channel with reproducible builds.

## iOS is mostly already served

Apple blocks nearly everything this needs: no IMEI since iOS 11, no MAC since iOS 7,
no other-app advertising IDs, no installed-app enumeration.
`NEPacketTunnelProvider` can run a local VPN, but App Store review is hostile to
tracker-inspection VPNs from unknown vendors.

More to the point, **App Privacy Report already ships in iOS** and shows per-app
network contacts and sensor access. Duplicating it badly is not worth the review
fight. Either ship a reduced build covering only permission audit and advertising-ID
state, or ship nothing on iOS and document why, pointing users at the built-in
report. The second is the honest default.
