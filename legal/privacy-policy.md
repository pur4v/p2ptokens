# Privacy Policy

**[Legal Entity Name]** — operator of the p2ptokens Peer-to-Peer AI Inference Network
**Effective Date:** [Effective Date] | **Last Updated:** [Last Updated]

> **DRAFT — FOR ATTORNEY REVIEW.**
> Prepared to reduce review burden; [Legal Entity Name] should still obtain a
> final legal review before publishing.

## 1. Introduction

[Legal Entity Name] ("we," "us," "the Platform") operates **p2ptokens**, a
peer-to-peer network that connects users who need AI inference ("Requesters")
with users who contribute compute — either local GPU hardware or third-party
model endpoints they control ("Providers"). This Policy explains what data we
process and your rights.

**Architectural reality (read Section 4):** inference is performed on hardware
operated by independent Providers, **not by us**. We operate a coordinator that
brokers matches but is **content-blind** — it never receives your prompts or
outputs.

## 2. Data We Process

### 2.1 Pseudonymous identity
- A **cryptographic public key (ed25519)** that serves as your peer id. This is
  generated locally on your device.
- **In v1 we do not collect names, email addresses, passwords, or account
  profiles.** *Planned v2:* if we introduce paid credits, accounts, payouts, or
  legally required identity verification (KYC/AML), we will collect and process
  the associated data and update this Policy first.

### 2.2 Network / coordinator metadata
- Multiaddresses, which **contain IP addresses and approximate location** (used
  for routing, dial-ability, latency, and sanctions screening).
- Advertised model identifiers, capacity, and availability.
- The **barter ratio ledger**: cumulative served/consumed token counts and a
  reputation score per peer id.
- Job metadata: timestamps, token counts, and which peers were matched.

### 2.3 Requester workload content (prompts, inputs, outputs)
- Transmitted **peer-to-peer and encrypted in transit**; the coordinator does
  **not** receive it. It is processed by the selected Provider (Section 4).

### 2.4 Peer-to-peer connection data
- Because peers connect directly, a Requester and a Provider generally **observe
  each other's IP address** (or the IP of a relay node used to traverse NAT).

### 2.5 Website, dashboard, and device data
- On p2ptokens.com and the local dashboard: **cookies and similar technologies**
  (see Section 5 and the [Cookie Policy](cookie-policy.md)), device/browser
  information, application logs, error reports, and diagnostics.

## 3. How We Use Data and Legal Bases

We process data to: operate and route jobs; match Requesters with Providers;
maintain the barter ratio and reputation; prevent fraud, abuse, and AUP
violations; comply with law (including sanctions/export screening); secure and
improve the service; and communicate service updates.

**Legal bases (GDPR/UK GDPR):** performance of contract (Art. 6(1)(b)) for core
service; legitimate interests (Art. 6(1)(f)) for security, integrity, and
analytics; legal obligation (Art. 6(1)(c)) for tax/sanctions where applicable;
consent (Art. 6(1)(a)) for optional cookies/marketing.

## 4. Peer-to-Peer Processing — What You Must Understand

This is the most important part of this Policy.

**4.1 Third-party hardware.** When you submit a job, your prompt and any attached
data are transmitted to one or more independent Provider nodes for processing.
Providers are not our employees or data centers.

**4.2 Technical safeguards we actually use (v1).**
- Prompts/outputs are **encrypted in transit** (libp2p Noise) between peers and
  across any relay; relay operators carry only ciphertext they cannot read.
- The **coordinator is content-blind** — it processes metadata only.

**4.3 What we do NOT yet do — read carefully.** To run a model, a Provider's
machine must process your prompt **in plaintext**. **In v1 there is no technical
mechanism that prevents a Provider from reading, logging, or retaining your
prompts and outputs.** Our prohibition on doing so is **contractual only** (see
the [Provider Agreement](provider-agreement.md)). Confidential-compute (TEE)
nodes and sandboxed execution are **planned, not currently available.**

**4.4 Data-minimization rule — what you must not submit.** Because Providers
process your prompts **in plaintext with no technical isolation in v1**, a
malicious or compromised Provider could read or retain data it processes. To keep
this risk within acceptable bounds, **you must not submit**:

- **(a) other people's personal data** — only submit your own content;
- **(b) special-category or sensitive personal data** (e.g., health, biometric,
  genetic, racial or ethnic origin, sexual orientation, religious or political
  beliefs); or
- **(c) regulated data**, including protected health information (PHI/HIPAA),
  financial data under GLBA, or cardholder data under PCI DSS.

Your own **non-sensitive content is fine.** Do not submit trade secrets or other
data you cannot tolerate being exposed. These restrictions apply unless and until
a designated confidential-compute tier is offered and you have executed any
required agreements with us.

**4.5 Roles under GDPR.** For **prompt/response content**, the Operator is
**neither controller nor processor**: it is content-blind and never receives that
content. The **Requester is the controller** of the content they submit, and the
**serving Provider is an independent processor or controller** of that content
under the [Data Processing Addendum](provider-agreement.md). The **Operator is
the controller** for connection and metadata data, IP addresses, website and
telemetry data, and (in planned v2) account and payment data. *If this allocation
is challenged, we will confirm the analysis with counsel for the specific
processing at issue.*

## 5. Cookies and Similar Technologies

We use cookies and local storage for essential functionality (e.g., recording
your **age confirmation** and **cookie choices**) and, where you consent, for
analytics. Non-essential cookies are set **only after you consent** via our
banner; you can withdraw consent at any time. Details, categories, and controls
are in the **[Cookie Policy](cookie-policy.md)**.

## 6. Data Sharing

We share data with: **Providers** (only the workload data needed to run your job,
peer-to-peer — Section 4); infrastructure, security, and analytics vendors acting
as our processors; sanctions/export-screening vendors; law enforcement or
regulators where legally required; and a successor in a merger or acquisition.
*Planned v2:* payment and payout processors and tax-reporting recipients. **We do
not sell personal information** and we do not "share" personal information for
cross-context behavioral advertising, each as those terms are defined under
CCPA/CPRA. We use **no advertising cookies**.

## 7. International Transfers

The network is global; your data (including your IP address and, for Providers,
workload content) may be processed in other countries, including on Provider
nodes worldwide. For EU/UK data transferred to countries without an adequacy
decision, we rely on the applicable Standard Contractual Clauses (and, for UK
data, the UK Addendum) together with supplementary measures. Region-pinning,
which will let Requesters restrict jobs to Providers in specified regions, is a
**planned feature**.

## 8. Retention

- **Coordinator metadata (registry, ratio ledger):** ephemeral and held
  in memory only — **not persisted in v1** (lost on restart).
- **Workload content (prompts/outputs):** **not stored by us** — we never receive
  it. Providers are contractually barred from retaining it (Section 4.3).
- **Website and application logs, diagnostics, and security logs:** 12 months.
- *Planned v2:* account, payment, and tax records — retained for 7 years.

## 9. Your Rights

Subject to your jurisdiction, you may access, correct, delete, port, or restrict
processing; object to processing based on legitimate interests; withdraw consent;
and opt out of "sale"/"sharing"/targeted advertising (CCPA/CPRA — noting that we
do not sell or share personal information). Contact privacy@p2ptokens.com. You may
complain to your supervisory authority (EU/UK) or attorney general (US states).

**P2P and pseudonymity limits.** Because identities are cryptographic keys and we
hold little directly identifying data in v1, we may be unable to link a request
to you, or may need additional information to verify a rights request. We will
delete data within our control; workload content is not held by us, and where a
Provider breaches its no-retention obligation, our practical ability to compel
deletion is limited to contractual remedies.

## 10. Security

We use administrative, technical, and organizational measures appropriate to the
risk (transport encryption, content-blind coordination, access controls, and
periodic security reviews). No system is perfectly secure; see Section 4.4. You
can report a security concern to security@p2ptokens.com.

## 11. Age Restriction — 18+

The Platform is intended **only for users aged 18 or older** (or the age of
majority in your jurisdiction, if higher). We present an **age-confirmation gate**
and do not knowingly collect data from, or provide the service to, anyone under
18. If we learn we have done so, we will terminate access and delete the data. See
also the Terms of Service eligibility clause.

## 12. Open Source and Self-Hosting

The p2ptokens client and coordinator are open source. Third parties may run their
own coordinators, relays, and clients that we do not operate or control. **This
Policy governs only the instance operated by [Legal Entity Name]** (e.g.,
p2ptokens.com). Data practices of independently operated nodes/instances are the
responsibility of their operators.

## 13. Changes

We will post material changes and provide at least **30 days' notice** via the
website or in-app notice before they take effect. This Policy is governed by the
laws of the jurisdiction in which the Operator is established.

## 14. Contact

Data Controller: [Legal Entity Name], [registered address]
Privacy contact / DPO: privacy@p2ptokens.com
Security contact: security@p2ptokens.com
Legal contact: legal@p2ptokens.com
EU/UK Representative (Art. 27 GDPR / UK GDPR): [EU/UK Representative name & address]
