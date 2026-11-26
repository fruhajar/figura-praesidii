//! Rendering. The evidence table is the product: a verdict the user cannot check is
//! worth nothing to them, and worth less in a letter to a controller.

use std::fmt::Write as _;

use fp_core::{Attribute, Identity, Remedy};
use fp_correlate::{Verdict, weights::Level};
use fp_lookup::Report;
use fp_request::Request;

pub fn render(report: &Report, id: &Identity, locality: Option<&str>, today: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Figura Praesidii, self-lookup");
    let _ = writeln!(
        out,
        "Subject: {}{}",
        id.names
            .iter()
            .map(|n| n.raw.as_str())
            .collect::<Vec<_>>()
            .join(" / "),
        id.birth_date
            .map(|d| format!(", born {}", d.to_cz()))
            .unwrap_or_default()
    );
    let scope = locality
        .map(str::to_string)
        .or_else(|| id.locality_tokens().first().cloned())
        .unwrap_or_else(|| "whole country".into());
    let _ = writeln!(out, "Locality searched: {scope}");
    let _ = writeln!(out, "Run on {today}, {} request(s) sent", report.requests);
    let _ = writeln!(out);

    section(&mut out, report, Verdict::Match, "YOURS");
    section(&mut out, report, Verdict::Possible, "POSSIBLY YOURS");

    let _ = writeln!(out, "Ruled out");
    let _ = writeln!(
        out,
        "  {} record(s) carried your name but contradicted you on a field that does not\n  \
         vary (date of birth, or a given name in a source that holds legal names). They\n  \
         belong to someone else, so they were counted and discarded, never stored.",
        report.excluded
    );
    let _ = writeln!(
        out,
        "  {} record(s) had too little in common to report.",
        report.dismissed
    );
    if report.dump_seen > 0 {
        let _ = writeln!(
            out,
            "  Read {} person record(s) out of the register's open data; {} carried \
             another\n  surname and were dropped without ever being kept.",
            report.dump_seen, report.dump_blocked
        );
    }
    let _ = writeln!(out);

    if !report.truncated.is_empty() {
        let _ = writeln!(out, "Stopped early");
        for t in &report.truncated {
            let _ = writeln!(out, "  {t}");
        }
        let _ = writeln!(
            out,
            "  A size limit ended the read, so finding nothing here rules nothing out.\n  \
             Raise max_records or max_bytes on the source, or use a narrower dump."
        );
        let _ = writeln!(out);
    }

    if !report.skipped.is_empty() {
        let _ = writeln!(out, "Not queried");
        for s in &report.skipped {
            let _ = writeln!(out, "  {s}");
        }
        let _ = writeln!(out);
    }
    if !report.errors.is_empty() {
        let _ = writeln!(out, "Failed");
        for (source, err) in &report.errors {
            let _ = writeln!(out, "  {source}: {err}");
        }
        let _ = writeln!(out);
    }
    if !report.unverified.is_empty() {
        let _ = writeln!(out, "Unchecked sources: {}", report.unverified.join(", "));
        let _ = writeln!(
            out,
            "  Their URL templates and field labels have never been run against the live\n  \
             site, so an empty result from them is not evidence of anything. Check them\n  \
             with `fp verify-sources` from a machine that can reach the hosts."
        );
        let _ = writeln!(out);
    }

    out.push_str(NEXT_STEPS);
    out.push_str(OFFER);
    out
}

fn section(out: &mut String, report: &Report, verdict: Verdict, heading: &str) {
    let findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.linkage.verdict == verdict)
        .collect();
    let _ = writeln!(out, "{heading} ({})", findings.len());
    if findings.is_empty() {
        let _ = writeln!(out, "  nothing");
        let _ = writeln!(out);
        return;
    }

    for (i, f) in findings.iter().enumerate() {
        let _ = writeln!(
            out,
            "\n[{}] {} ({})",
            i + 1,
            f.source.title,
            f.source.kind.label()
        );
        if let Some(locator) = &f.record.locator {
            let _ = writeln!(out, "    {locator}");
        }
        let _ = writeln!(
            out,
            "    confidence {:.0} bits, of which {:.0} can tell you from a namesake{}",
            f.linkage.bits,
            f.linkage.discriminating_bits,
            if f.verified {
                ""
            } else {
                " [source unchecked]"
            }
        );

        // A field worth no bits is context, not evidence: it goes below with the rest
        // of what the page published.
        let (scored, unscored): (Vec<_>, Vec<_>) = f
            .linkage
            .evidence
            .iter()
            .partition(|e| e.bits != 0.0 || e.level == Level::Conflict);

        let _ = writeln!(out, "    evidence");
        for e in scored {
            let _ = writeln!(
                out,
                "      {:<14} {:<8} {:>5.1}  yours: {}  |  published: {}{}",
                e.field.label(),
                e.level.label(),
                e.bits,
                truncate(&e.claimed, 28),
                truncate(&e.published, 36),
                e.note
                    .as_deref()
                    .map(|n| format!("  ({n})"))
                    .unwrap_or_default()
            );
        }
        if let Some(veto) = &f.linkage.veto {
            let _ = writeln!(out, "      ruled out: {veto}");
        }

        let mut extra: Vec<String> = f
            .record
            .attrs
            .iter()
            .filter_map(|a| match a {
                Attribute::Other { label, value } => Some(format!("{label}: {value}")),
                _ => None,
            })
            .collect();
        extra.extend(
            unscored
                .iter()
                .map(|e| format!("{}: {}", e.field.label(), e.published)),
        );
        if !extra.is_empty() {
            let _ = writeln!(out, "    also published: {}", extra.join("; "));
        }

        let p = &f.protection;
        let _ = writeln!(
            out,
            "    protection: {} ({})",
            p.remedy.label(),
            p.articles.join(", ")
        );
        for line in wrap(p.rationale, 72) {
            let _ = writeln!(out, "      {line}");
        }
        if p.remedy != Remedy::Immovable {
            let _ = writeln!(
                out,
                "      Reply due in {} days, extensible by {}. No reply: complain to {}.",
                p.response_days,
                p.extension_days,
                p.authority.unwrap_or("the supervisory authority")
            );
        }
        if let Some(controller) = &f.source.controller {
            let _ = writeln!(
                out,
                "      Address it to: {}{}",
                controller.name,
                controller
                    .email
                    .as_deref()
                    .map(|e| format!(" <{e}>"))
                    .unwrap_or_default()
            );
        } else {
            let _ = writeln!(
                out,
                "      Recipient unknown: take it from the site's imprint."
            );
        }
        if let Some(from) = &f.source.copies_from {
            let _ = writeln!(
                out,
                "      Copies {from}, which is the weakness: a mirror cannot claim the\n      \
                 statutory duty that protects the register itself."
            );
        }
    }
    let _ = writeln!(out);
}

const NEXT_STEPS: &str = "\
What to do with this
  1. Open every locator above and check the evidence by eye. A tool that says \"yours\"
     without showing why is guessing on your behalf.
  2. Run `fp requests` to draft the letter for each finding, grounded on the route
     that target actually admits.
  3. Where the route is \"immovable\", do not send an erasure request: it will be
     refused, and the refusal costs you a month. Send the Article 15 access request
     instead and keep the answer. It is the evidence base for every mirror.
  4. Search your own name and note which mirrors rank above the register. Those are
     the entries worth a request, and the ones this run cannot find for you: add them
     to the catalog as commercial_mirror entries.
  5. Keep this report off any synced folder. It is a dossier on you, assembled.
";

/// The paid half, stated as plainly as the free half. The contact line is left unset on
/// purpose: the licensing split for this project is not settled.
const OFFER: &str = "\n\
A detailed review goes further
  This run covers public registers in one locality, correlated on name, date of birth
  and address, with a drafted request per finding. It does not cover:
    - breach and broker exposure (HIBP, people-search sites, bidstream brokers)
    - the mirror sweep: finding every site that republishes your register entry
    - deadline tracking and escalation to ÚOOÚ when a controller ignores you
    - what your phone emits: advertising ID, tracker endpoints, Wi-Fi and BLE
      broadcasts, and which apps hold location and contacts
  Contact: unset in this build.
";

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let kept: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}~")
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Letters, each preceded by what has to be checked before it is sent. The warnings
/// stay out of the letter body so that a file written with `--out-dir` is the letter
/// and nothing else.
pub fn render_requests(requests: &[Request], held_back: usize, report: &Report) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Figura Praesidii, drafted requests\n");

    if requests.is_empty() {
        let _ = writeln!(
            out,
            "Nothing to draft. {} finding(s) in the last scan, {} of them not confident\n\
             enough to write to a controller about.",
            report.findings.len(),
            held_back
        );
        return out;
    }

    for (i, r) in requests.iter().enumerate() {
        let _ = writeln!(out, "{}", "=".repeat(78));
        let _ = writeln!(
            out,
            "[{}] {} -> {}",
            i + 1,
            r.source_id,
            r.recipient
                .as_ref()
                .map(|c| c.name.as_str())
                .unwrap_or("recipient unknown")
        );
        if let Some(locator) = &r.locator {
            let _ = writeln!(out, "    about: {locator}");
        }
        let _ = writeln!(
            out,
            "    reply due in {} days, extensible by {}",
            r.deadline_days, r.extension_days
        );
        let _ = writeln!(out, "    file: {}", r.filename());
        for w in &r.warnings {
            for (n, line) in wrap(w, 70).into_iter().enumerate() {
                let _ = writeln!(
                    out,
                    "    {} {line}",
                    if n == 0 { "CHECK:" } else { "      " }
                );
            }
        }
        let _ = writeln!(out, "{}\n", "-".repeat(78));
        let _ = writeln!(out, "{}", r.body);
    }

    let _ = writeln!(out, "{}", "=".repeat(78));
    if held_back > 0 {
        let _ = writeln!(
            out,
            "\n{held_back} finding(s) held back as only possibly yours. Pass\n\
             --include-possible to draft those too, after reading their evidence in\n\
             `fp scan`."
        );
    }
    let _ = writeln!(
        out,
        "\nThese are templates, not legal advice. Read each one, fill in the bracketed\n\
         placeholders, and keep a copy plus the date you sent it: the statutory clock\n\
         starts on delivery."
    );
    out
}
