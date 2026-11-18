# Figura Praesidii: desktop

The tool itself. Everything the user actually runs lives here.

## Pipeline

```
adapters ──► findings store ──► request generator ──► ICS deadlines
(sources)    (encrypted,        (per-target legal
              local)             grounds)
```

## Slice 1: public register exposure

Czech registers publish more than people expect. `or.justice.cz` carries company
officers' full name, date of birth and **home address**; RŽP exposes sole traders'
places of business, usually also home; `isir.justice.cz` covers insolvency; the land
registry covers property.

**Three target classes, three different legal routes.** Getting this right per target
is the product.

| Target | Route | Outcome |
|---|---|---|
| The register itself | None available. Publication is a statutory obligation, so Art. 17(3)(b) defeats erasure. | Immovable. Say so plainly rather than generating a letter that will fail. |
| Commercial mirror sites | Art. 21 objection to processing based on legitimate interest, then Art. 17(1)(c) erasure. Their balancing claim is far weaker than the register's. | Removable. This is where the value is. |
| Search engines | Art. 17 delisting, on *Costeja* and *GC v CNIL*. | Delistable. Often what the user actually wants, even when the source stays up. |

Enumerate the footprint, identify which mirrors carry it, generate the correctly
grounded request for each. Hand-verifiable against the source, which is the whole
appeal: no corpus, no API keys, no trust required in our data.

**Be a polite client.** Querying a public register about yourself is fine; hammering
it is not. Rate-limit, identify the client honestly, cache aggressively.

## Slice 2: breach and broker exposure

HIBP for breach hits by email and phone, with the user's own API key. Then the part
with the real value: knowing *which* holders to ask, generating the requests, and
tracking the one-month deadline (extensible to three) with escalation to ÚOOÚ when a
controller ignores it.

## Storage

Encrypted at rest under a user passphrase. Not synced by default. API keys in the OS
keyring, never in a config file.

This matters more than usual: **the findings store is a ready-made dossier on the
user**, which makes it the highest-value file on their machine. Saving a report is
explicit, never automatic, and the export path should run through Figura Munda before
anything is shared.

## Out of scope

Any adapter whose input is *someone else's* identifier. Holehe-style enumeration
reports which sites an arbitrary address is registered on, which quietly turns this
into a lookup tool for third parties. Either bind such adapters behind proof of
control of the address, or leave them out. Do not ship them unguarded and describe
the result as self-OSINT.
