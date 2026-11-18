# Figura Praesidii — online

The most constrained surface in the project. Static by default.

## The invariant

**The server must never receive an identifier being looked up.** Not a name, not an
email, not a birth date, not a register number. All lookups happen in the desktop
client, against the source, with the user's own keys.

The test for every proposed feature: *if this server were seized or compromised
tomorrow, what would the attacker learn about any user's exposure?* The required
answer is "which version of the adapter file they downloaded, and from which IP."
Anything that changes that answer does not belong here.

## What it serves

1. **Documentation.** What the tool does, what it cannot do, and the legal grounds
   behind each request type.
2. **Request templates.** Article 15 (access) and Article 17 (erasure) letters in
   Czech, Slovak and English, plus the Article 21 objection, each with the grounds
   filled in per target class.
3. **The adapter file.** Register sources and the mirror-site registry, versioned and
   signed, pulled by the client. Signing matters: a tampered adapter file could
   redirect a user's queries.

## The mirror-site registry is worth publishing as open data

"Which sites republish Czech public-register data, and which fields each carries" is a
useful public-interest dataset in its own right, maintained by the project anyway.
Publishing it openly costs nothing, invites correction, and builds the credibility
this project needs.

## No deadline tracker. Ship an ICS export instead.

A hosted tracker for the one-month statutory response window sounds harmless and is
the one feature that would give this server state about real people's requests. It is
not needed: the client can emit a calendar file, and the user's own calendar does the
reminding, on every device, with no server knowing a request was ever made.

If cross-device email reminders are ever genuinely demanded, the stored record must be
an opaque client-generated id plus a due date plus a contact address, and never the
controller's name, the subject's name, the identifier, or any finding.

## Known leak, stated honestly

Fetching the adapter file reveals to the server that a given IP runs this tool. That
is not nothing. Mitigate by bundling adapters in releases so updating is optional,
serving over a CDN, and keeping the fetch functional over Tor. Do not pretend it is
zero.

## Explicitly out of scope

No accounts. No lookups or query proxying. No server-side correlation. No analytics
that could reveal what was searched, including page-level analytics on documentation
pages tied to specific registers. No logs beyond what a static host unavoidably
writes, with retention set as short as the host allows.
