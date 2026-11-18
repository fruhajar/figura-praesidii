# Figura Praesidii

*Praesidii*, the genitive: "figure of protection". *Praesidium* is neuter nominative
and would not agree.

Shows you what the internet holds about *you*, then helps you get it removed.

You run it on yourself. It reports your own exposure and generates the GDPR Article
15 (access) and Article 17 (erasure) requests for each holder it finds, then tracks
the statutory response deadline.

> **Status:** nothing built. This directory holds the plan.

## Self-lookup only

This is a hard architectural boundary, not a policy note, and it is recorded here so
it does not creep back in later.

Looking up **other people** is out of scope. Two features in particular are
permanently excluded: searching by *description* rather than by a known identifier,
which makes the tool a reverse-identification engine aimed at someone who has not
said who they are; and inferring **"people this person knows,"** since a social graph
around a private individual is the working core of stalking and harassment. "It is
all public" does not change this, because the harm lives in the aggregation. Public
facts become a targeting package once assembled.

Hosting that version would also make the operator a GDPR controller with
unsatisfiable duties: no Article 6 basis for profiling third parties for anonymous
requesters, Article 14 notification that kills the product if honoured and is the
breach if not, Article 9 special categories swept in unavoidably through the graph,
mandatory DPIA. *Clearview AI* is the settled precedent across the French, Italian,
Greek and Dutch authorities, where "the data was already public" lost every time and
the Dutch DPA warned of personal liability for directors.

Inverting the direction fixes all of it at once. The data subject becomes the
consenting user, and the hard articles turn from liability into the feature set.

For the investigator-side use case, Maltego, SpiderFoot, OCCRP Aleph, theHarvester,
Holehe and Sherlock already exist, and all of them ship as locally-run tools
precisely so the controller liability sits with the searcher.

## Three slices

### 1. Public register exposure (start here)

Czech public registers expose far more than people realise. `or.justice.cz` publishes
company officers' full name, date of birth and **home address**, so anyone who has
ever been a jednatel has their residence online. RŽP exposes sole traders' places of
business, usually also home. Add `isir.justice.cz` (insolvency) and the land registry
and the picture gets detailed.

**The asymmetry that makes this a product:** the register entry itself is immovable,
since publication is a statutory obligation and an Article 17 request against it will
fail. The commercial mirror sites that scrape the register, republish it and rank on
people's names have a far weaker legitimate-interest claim. That is where erasure
requests actually bite, and the mirror is what surfaces when someone searches their
own name.

Enumerate the footprint, find the mirrors carrying it, generate the correctly-grounded
Czech-language request for each. Novel, lawful, no corpus needed, and hand-verifiable
against the source.

### 2. Breach and broker exposure, with the request pipeline

HIBP for breach hits by email and phone. Then the part that is actually valuable:
knowing *which* holders to ask, generating the requests, tracking the one-month
deadline (extensible to three), and escalating to ÚOOÚ when a controller ignores it.

### 3. Device emission measurement (v2, mobile)

See below. This replaces the device-ID lookup idea, which is not buildable.

## Device-ID search does not work, and why

The original idea was querying by personal device IDs on the reasoning that a
stranger would not know them, making the tool self-limiting by construction. The
reasoning is sound but the identifiers do not hold up, and there is nothing to query.

| Identifier | Actually secret? |
|---|---|
| IMEI, serial | Yes. Needs the device, the box, or the carrier bill. |
| Wi-Fi / BLE MAC | **No.** Broadcast over the air; capturable by a neighbour or anyone who shared a café. OS randomisation helps, but the real address leaks on association. |
| Advertising ID (GAID / IDFA) | **No.** Every ad SDK in every installed app receives it, and it is actively traded in bidstream data. |

And the corpus problem is fatal. Indexing device IDs means buying breach dumps or
bidstream and location broker data, which would make this project the holder of
exactly the dataset it exists to oppose, on millions of non-consenting people, while
funding the harm. There is no lawful, accessible corpus.

**The buildable inversion is local measurement.** Do not ask a database what it knows
about the device; measure what the device emits. Which advertising ID it currently
presents, which trackers and SDK endpoints it contacts and how often, what it
broadcasts over Wi-Fi and BLE when idle, whether MAC randomisation is actually on,
which apps hold location, contacts and clipboard access and when they last used them.

No corpus, no API keys, no broker money. Findings are actionable the same minute, and
it reports measured reality rather than a probabilistic guess about identity.

## Verification scales with what is revealed

| Reveals | Verification |
|---|---|
| Public register data | **None needed.** Nothing is disclosed that is not already public. |
| Breach hits for an email or phone | Emailed or texted code. HIBP's model: open lookup permitted, sensitive breaches gated on proof of control of the address. |
| Device identifiers and emissions | Companion app reading its own identifiers. Typing an IMEI proves nothing; running code on the handset proves a lot. |
| Anything at all | **Never accept ID document scans.** It would make this the custodian of the highest-value data it could hold, and it is what age-verification vendors keep getting breached over. |

## Layout

Each slice has its own scope document. Read those before touching code.

- **[desktop.md](desktop.md)** the tool itself, slices 1 and 2. The main deliverable,
  and the only one with code behind it: see [architecture.md](architecture.md).
- **[online.md](online.md)** deliberately the most constrained surface in the whole
  project. Static by default, and it must never receive an identifier being looked up.
- **[mobile.md](mobile.md)** slice 3, device emission measurement. Android is the real
  target; iOS blocks most of what this needs and Apple already ships much of it.

## Why local-first settles the resource question

If the tool runs on the user's machine, their queries never touch the server. Nothing
to jam up, because there is no shared compute, and no third-party data is processed,
which removes most of the compliance burden. Same reasoning that keeps Maltego and
SpiderFoot local.

## Two residual risks

1. **The output is a ready-made dossier on the user**, which makes the report the
   highest-value file on their disk. Encrypt at rest, never sync it, make saving
   explicit rather than default. It should pass through Figura Munda before being
   shared with anyone.
2. **Some adapters accept arbitrary input.** Holehe-style enumeration will report
   which sites *any* address is registered on, so the local build is not fully
   abuse-proof. Either bind those adapters behind the verified tier or state the gap
   plainly in the README. Do not claim an abuse-proof local tool.

## Related but deliberately separate

Transparency obligations for public officials, company directors and beneficial
owners are a real and defensible category, with the opposite default posture, and
they belong in a different product. Note that even there the trend is restrictive:
*WM and Sovim* (CJEU C-37/20, November 2022) struck down public access to
beneficial-ownership registers, limiting it to those showing a legitimate interest.
OpenCorporates, OpenSanctions and OCCRP Aleph already cover the space.
