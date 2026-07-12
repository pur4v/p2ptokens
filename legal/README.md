# p2ptokens — Legal Documents

> ## ⚠️ These are DRAFTS. Not legal advice.
> These documents were prepared as engineering-informed drafts to be handed to
> **qualified legal counsel**. They have **not** been reviewed by a lawyer.
> **No document can guarantee "no legal issues."** Do not publish or rely on any
> of these until counsel has reviewed and localized them for every jurisdiction
> in which you operate. Every `[bracketed]` value and every "Counsel note" must
> be resolved before publication.

## Contents

| File | Purpose |
| :--- | :--- |
| [privacy-policy.md](privacy-policy.md) | What data is collected/processed and users' rights |
| [cookie-policy.md](cookie-policy.md) | Cookies / local storage used, and consent |
| [terms-of-service.md](terms-of-service.md) | The agreement governing use of the network |
| [acceptable-use-policy.md](acceptable-use-policy.md) | Prohibited content and conduct |
| [provider-agreement.md](provider-agreement.md) | Terms for users who contribute compute |
| [exhibit-a-dpa.md](exhibit-a-dpa.md) | Data Processing Addendum (GDPR Art. 28) |
| [exhibit-b-hardware-security-specs.md](exhibit-b-hardware-security-specs.md) | Provider hardware & security minimums |
| [exhibit-c-fees-payouts.md](exhibit-c-fees-payouts.md) | Fee/payout table (planned, v2) |
| [FINALIZATION-CHECKLIST.md](FINALIZATION-CHECKLIST.md) | **How to complete these without a lawyer + irreducible risks** |

A cookie-consent banner and an **18+ age gate** are implemented in the dashboard
(`crates/client/src/ui.html`) and should also front the public site.

## Ground truth — what p2ptokens actually does (v1)

The legal drafts are written to reflect these facts accurately. **If the
architecture changes, update these documents**, because accuracy is the primary
legal protection (overstating protections invites enforcement).

1. **Pseudonymous identity.** A peer is an anonymous **ed25519 keypair**; the
   public key is the peer id. v1 has **no accounts, names, emails, passwords, or
   KYC**.
2. **Barter economy, no money (v1).** Access is earned by serving (an
   upload/download **ratio**), not purchased. **Paid credits, payouts, tax
   reporting, and KYC are a planned v2** — those sections are retained and marked
   accordingly.
3. **Coordinator is content-blind.** The central coordinator (tracker) sees only
   **metadata** — peer ids, multiaddrs (which contain IP addresses), model names,
   token counts, and the ratio ledger. It **never** receives prompt/response
   content.
4. **Inference bytes are peer-to-peer and encrypted in transit** (libp2p Noise).
5. **Providers see plaintext.** To run a model the provider's machine
   necessarily processes the prompt in cleartext (homomorphic encryption is not
   viable). **In v1 there is NO technical mechanism preventing a provider from
   reading, logging, or retaining prompts/outputs — the prohibition is
   contractual only.** Confidential compute (TEE) and sandboxed execution are
   **roadmap (v2)**, not present in v1. Documents must not claim otherwise.
6. **Peers learn each other's IP addresses.** Because connections are direct P2P
   (or via a relay whose operator sees the encrypted stream), a Requester and
   Provider generally observe each other's IP (or the relay's).
7. **BYO third-party backends.** A provider may serve local open models (Ollama)
   or proxy a third-party endpoint using **their own credentials**. Proxying paid
   third-party API access may violate that provider's terms — the provider bears
   that risk and must have the right to do so.
8. **Open source / self-hosting.** The client and coordinator are open source;
   third parties can run their own instances. These documents govern only the
   instance operated by [Legal Entity Name].

## Placeholders to resolve before publishing

The drafts have been **hardened** — every "Counsel note" is now a resolved
in-text decision and all operational values are filled. The **only** brackets
left are facts only you have:

- `[Legal Entity Name]`, `[registered address]`
- `[Effective Date]`, `[Last Updated]`
- `[EU/UK Representative name & address]` (only if you serve EU/UK with no EU/UK
  establishment)

Plus two actions (not text edits): **register a DMCA agent** with the U.S.
Copyright Office, and **attach the SCC/UK-IDTA annexes** if you transfer
restricted EU/UK data. See [FINALIZATION-CHECKLIST.md](FINALIZATION-CHECKLIST.md)
for the decisions that were made and the realistic no-lawyer review options.
