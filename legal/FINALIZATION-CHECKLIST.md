# Finalization Checklist

> **Read this first.** These documents are **drafts, not legal advice.** I (the
> tool that generated them) am not a lawyer and cannot certify them or guarantee
> you have "no legal issues." This checklist gets you as far as a non-lawyer
> reasonably can, and points to realistic, low-cost ways to get real review. Some
> risk in this business model is **irreducible by documents alone** (see §4).

## 1. Values only you can fill in (this is now the whole list)

The drafts have been hardened so that **the only brackets left to replace are
facts only you have.** Search `legal/` for `[` — you should find only:

- **`[Legal Entity Name]`** and **`[registered address]`** — you need an actual
  legal entity (LLC/Ltd/etc.). Operating a paid, data-processing, global service
  as an individual is a serious personal-liability risk. **Form an entity first.**
- **`[Effective Date]` / `[Last Updated]`** — the dates you publish.
- **`[EU/UK Representative name & address]`** — only if you target/monitor EU/UK
  users and have no establishment there (appoint an Art. 27 representative).

Everything else is already decided in-text (see §2).

## 2. Decisions already made for you (previously "Counsel notes")

These were resolved with defensible, conservative defaults so a lawyer only has
to *sanity-check* them, not author them:

- **GDPR roles** — stated: Operator is content-blind → **neither controller nor
  processor of prompt content**; Requester is controller; serving Provider is the
  processor (DPA, Exhibit A). Operator is controller for metadata/IP/telemetry
  and (v2) account/payment data.
- **Governing law & disputes** — keyed to "the jurisdiction in which the Operator
  is established"; binding **individual arbitration + class-action waiver** with
  **small-claims and EU/UK consumer carve-outs** (no local-court deprivation).
- **Liability caps** — ToS: greater of 12-months-paid or **USD $100** (floors at
  $100 under v1 barter); Provider: 3-months-paid (**$0** in v1); statutory
  carve-outs retained.
- **Prohibited data (key risk-narrowing choice, now firm)** — users may **not**
  submit others' personal data, special-category/sensitive data, or regulated
  data (PHI/GLBA/PCI); their own non-sensitive content is fine. This is the
  compensating measure for the plaintext/no-sandbox reality and shrinks GDPR/CCPA
  scope.
- **CCPA/CPRA** — Operator does **not** sell or "share"; **no ad cookies**.
- **Retention** — coordinator metadata ephemeral; logs 12 months; (v2) records 7
  years.
- **Unused credits (v2)** — refunded/rolled over, **no forfeiture**.
- **Cookies** — strictly-necessary only by default; analytics behind opt-in.
- **SCCs** — EU SCCs (2021/914) + UK IDTA cited in the DPA; you still must
  **attach/complete the SCC annexes** if you transfer restricted EU/UK data.

You (or a lawyer) should still confirm these fit your actual markets — but they
are drafted, not open questions.

## 3. Concrete actions before any public launch

- [ ] Form a legal entity; use it as [Legal Entity Name].
- [ ] Register a **DMCA agent** with the U.S. Copyright Office (~$6, online) →
      fill ToS §9.
- [ ] Stand up the contact inboxes and actually monitor `security@`/`privacy@`.
- [ ] Implement **data-subject request** handling and **breach response** (you
      already have the 24h provider-breach clause).
- [ ] Make sure the **cookie banner** blocks non-essential cookies until consent
      (the dashboard's banner is a start; the public site needs the same).
- [ ] Add **sanctions/geo screening** if you enable payments or want to enforce
      the export clauses (IP-based blocking of embargoed regions).
- [ ] Keep the docs **in sync with the code** — if you add TEE/sandbox or
      payments, update the docs the same day (accuracy is your main protection).

## 4. Risk that documents do NOT fix (be honest with yourself)

- **Proxying paid third-party APIs** (OpenAI/Anthropic/etc.) to strangers
  violates those providers' terms and can get accounts banned; a clause shifting
  risk to the Provider reduces *your* exposure but does not make the underlying
  activity compliant. Consider disabling paid-API proxying and shipping
  **local-model (Ollama) sharing only** for v1.
- **Plaintext prompts on strangers' machines** is a genuine privacy exposure. The
  honest mitigation is telling users not to send sensitive data (done) and
  building the confidential-compute tier (roadmap) — not stronger legal wording.
- **Money/credits** (v2) can trigger money-transmission, e-money, tax-reporting,
  and (if transferable/tokenized) securities/MiCA regimes. Do not enable without
  specific advice.

## 5. Getting real review without a traditional lawyer

Realistic, founder-friendly options (not endorsements; verify current suitability):

- **Policy generators** built for non-lawyers — e.g. Termly, iubenda, Osano,
  GetTerms. They keep policies updated and handle cookie-consent tooling, **but**
  they're generic and will **not** understand the P2P/plaintext nuance — you must
  graft in your Section-4-style disclosures from these drafts.
- **Flat-fee / subscription legal** — Rocket Lawyer, LegalZoom, or
  Clerky/Stripe-Atlas legal partners give you attorney Q&A at low cost; good for a
  one-time review of these specific drafts.
- **Startup/small-business legal clinics** (many law schools) offer free or
  low-cost help.
- **Jurisdiction-specific services** — third parties that act as your GDPR
  **Art. 27 EU representative** / UK representative if you need one.

**Minimum viable posture if you truly cannot get review yet:** form an entity,
run **local-model sharing only** (no paid-API proxying), **prohibit personal
data** in the AUP/ToS, keep it **barter-only** (no money), register a DMCA agent,
and publish these drafts with the entity/contact/date fields filled. That
materially lowers — but does not eliminate — your exposure.
