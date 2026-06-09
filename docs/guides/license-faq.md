# License FAQ

This document answers the questions a contributor or consumer of the
SMSengine would have about licensing.

## TL;DR

The SMSengine is **dual-licensed under `MIT OR Apache-2.0`**. Both are
permissive open-source licenses. You can use, modify, redistribute, and
sell products that include the engine, for free, under either license
at your option. The canonical text files are:

- `LICENSE-MIT` — the MIT License (one of the two options)
- `LICENSE-APACHE` — the Apache License, Version 2.0 (the other option)
- `NOTICE` — the attribution file required by Apache 2.0 § 4d
- `LICENSE` — a symlink to `LICENSE-APACHE` (the Rust convention)

If you have a legal team, the **most defensible** choice is **Apache 2.0**
because it has an explicit patent grant and a clear attribution mechanism
(per `NOTICE`). If you want the absolute minimum boilerplate, choose
**MIT** (it has fewer obligations).

## Common questions

### Which license applies?

Both. The engine is dual-licensed. You (the consumer / contributor /
shipper) pick whichever license is friendlier to your legal team. If
your team has a policy that forbids one but not the other, you can
choose the other.

In practice:
- **Apache 2.0** is preferred if you want an explicit patent grant and
  clear attribution rules.
- **MIT** is preferred if you want the absolute minimum boilerplate
  (just the copyright + permission notice).

### Can I use the engine in a proprietary product?

Yes. Both MIT and Apache-2.0 allow proprietary use. You do **not**
need to open-source your application code that uses the engine. You
**do** need to retain the copyright and permission notices (per
MIT) or the NOTICE file attribution (per Apache 2.0 § 4d) in any
distribution that includes the engine's source or binary.

### Can I use the engine in a SaaS without contributing back?

Yes. Neither MIT nor Apache-2.0 has the AGPL-style "viral" clause that
requires SaaS consumers to open-source their stack. This is **unlike**
the AGPL-3.0, which would require SaaS consumers to publish their
modifications. The engine explicitly chose the more permissive
MIT/Apache path so that SaaS use is unrestricted.

### Do I need to attribute the engine?

Yes, in two ways:

1. **MIT requires** the copyright + permission notice to be included
   with any distribution that includes the engine's source.
2. **Apache 2.0 § 4d requires** a readable copy of the attribution
   notices in the `NOTICE` file, in the Derivative Works' NOTICE or
   documentation.

The `NOTICE` file in this repo contains the engine's attribution
("SMSengine, Copyright 2026 The SMSengine Authors, https://smsengine.dev").
When you distribute the engine (binary or source), include the
contents of `NOTICE` in your distribution.

### What if my company wants a warranty or indemnification?

MIT and Apache-2.0 are provided **"AS IS"** with **no warranty** and
**no indemnification**. Most open-source projects ship this way. If
your legal team requires warranty or indemnification, you can:

- Use a different open-source library that offers a warranty through
  its maintainer.
- Hire an independent consultant to provide a separate warranty
  agreement covering the engine's use in your deployment.
- Negotiate a private agreement with the entity that operates the
  engine for your deployment.

The project itself does not provide a separate warranty or
indemnification layer; the open-source licenses explicitly disclaim
all warranties to the maximum extent permitted by law.

### Will the license change in the future?

The engine's **current license is `MIT OR Apache-2.0`** and is
considered stable. Each existing release of the engine is under
that license and will remain so. Future releases may refine the
license metadata (clarifying SPDX expressions, adding copyright
attribution, etc.) but will not retroactively change the license of
any prior release.

If a future release does add a new licensing option, the new option
will be **additive** — adopters of the prior release keep their
existing license, and adopters of the new release choose from the
expanded set.

### Why MIT OR Apache-2.0 and not just one?

The Rust ecosystem convention is `MIT OR Apache-2.0`. Both are equally
permissive. The "OR" exists because:

- The Rust language itself is dual-licensed the same way.
- Cargo's `license` field accepts SPDX expressions like
  `MIT OR Apache-2.0` directly.
- The "OR" gives consumers flexibility if their legal team has
  policies that forbid one but not the other.

Some projects ship a single license (e.g. `MIT` only) to reduce
complexity. The engine chose the dual-license path to maximise
adoption.

### What about the third-party crates the engine depends on?

The engine's 27 third-party dependencies (per
[`docs/decisions/ADR-015-ExternalCrates.md`](../decisions/ADR-015-ExternalCrates.md))
are all permissively licensed (MIT, Apache-2.0, BSD-3, or dual-licensed).
No copyleft, no AGPL, no SSPL. The license hygiene policy in
ADR-015 enforces this on every new dependency.

### Can I sell products that include the engine?

Yes, under both MIT and Apache-2.0. The "permission notice" in
MIT explicitly says "to permit persons to whom the Software is
furnished ... to sell copies of the Software."

### Can I modify the engine and not publish my changes?

Yes. MIT and Apache-2.0 do not require you to publish your changes.
(If you redistribute the engine, you must retain the copyright
notices — but you don't have to publish your modifications.)

The one exception is **AGPL-3.0**, which would require you to publish
your modifications if you ship a SaaS based on the engine. The engine
deliberately chose MIT/Apache-2.0 over AGPL-3.0 to avoid this.

### What about trademarks?

Neither MIT nor Apache-2.0 grants trademark rights. You can use the
engine's source code, but you **may not use the SMSengine name and
logos** in your marketing materials without separate written
permission. Per Apache 2.0 § 6, the trademark restriction is
explicit.

## How is the license decision documented?

The license decision is documented in three places:

1. **This FAQ** (`docs/guides/license-faq.md`) — the consumer-facing
   license explanation.
2. **The README** — a one-paragraph "License" section with pointers
   to the LICENSE files.
3. **`docs/decisions/`** — the ADRs that document the engine's
   architectural decisions. ADR-015 (External Crate Selection)
   includes a license-hygiene policy.

## See also

- [`LICENSE-MIT`](../../LICENSE-MIT) — the MIT License text
- [`LICENSE-APACHE`](../../LICENSE-APACHE) — the Apache License 2.0 text
- [`NOTICE`](../../NOTICE) — the attribution file
- [`docs/decisions/ADR-015-ExternalCrates.md`](../decisions/ADR-015-ExternalCrates.md)
  — the dependency-license audit log
- [`CONTRIBUTING.md`](../../CONTRIBUTING.md) — the spec-to-PR workflow
- [`AGENTS.md`](../../AGENTS.md) — the engine's orientation for AI
  agents and human developers
