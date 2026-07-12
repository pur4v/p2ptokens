> **DRAFT — FOR ATTORNEY REVIEW.** Prepared to reduce review burden; [Legal Entity Name] should still obtain a final legal review before publishing.

# Terms of Service

**Operator:** [Legal Entity Name] ("**[Legal Entity Name]**", "**we**", "**us**", or "**our**")
**Product:** p2ptokens (the "**Platform**" or "**Service**")
**Domain:** p2ptokens.com
**Effective date:** [Effective Date]
**Last updated:** [Last Updated]

These Terms of Service (the "**Terms**") govern your access to and use of the instance of p2ptokens operated by [Legal Entity Name] at p2ptokens.com and its associated coordinator/matchmaking services. p2ptokens is open-source software that others may self-host; **these Terms govern ONLY the instance operated by [Legal Entity Name]**. If you use a third-party or self-hosted instance, your relationship is with that operator, not with us.

---

## 1. Agreement to These Terms

By accessing or using the Service, downloading or running the client, connecting a peer, or otherwise participating in the network operated by us, you agree to be bound by these Terms. If you do not agree, do not use the Service.

These Terms incorporate by reference, and you also agree to:

- our **[Privacy Policy](privacy-policy.md)**;
- our **[Cookie Policy](cookie-policy.md)**;
- our **[Acceptable Use Policy](acceptable-use-policy.md)** ("**AUP**"); and
- if you act as a **Provider** (a peer that serves inference to others), our **[Provider Agreement](provider-agreement.md)**, which contains additional obligations specific to serving.

If any conflict exists between these Terms and an incorporated document, these Terms control unless the incorporated document expressly states otherwise.

Acceptance is captured through a **click-through or run-time acceptance mechanism** presented before use, which records your agreement notwithstanding the pseudonymous, no-account model (see Section 3).

---

## 2. The Service

p2ptokens is an **intermediary, routing, and matchmaking layer** for peer-to-peer AI inference. We operate a **coordinator** that matches peers who want inference performed ("**Requesters**") with peers who offer to perform it ("**Providers**"), and that maintains a ledger of contribution (see Section 4).

You acknowledge and agree that:

- **Inference is performed by independent third parties.** The actual computation, model hosting, and generation of outputs are carried out by Providers who are independent third parties and **not** our employees, agents, partners, or subcontractors. We do not perform inference and do not control Providers.
- **We are a conduit, not a party to the exchange of content.** We facilitate connections; the substance of any request and response flows between peers.
- **The coordinator is content-blind.** The coordinator sees only **metadata** — for example peer identifiers (public keys), network addresses (multiaddrs, which include IP addresses), model names, token counts, and the contribution ledger. The coordinator **never receives the content of your prompts or the outputs** generated for you.
- **Inference traffic is peer-to-peer.** The bytes of your requests and responses travel directly between peers and are **encrypted in transit** using the libp2p Noise protocol. Encryption in transit does **not** mean the receiving Provider cannot read the content once decrypted at its node (see Sections 5 and 10).
- **No guarantees of availability, accuracy, or confidentiality.** We do **not** guarantee that the Service, the coordinator, or any Provider will be available, uninterrupted, timely, secure, or error-free; that any output will be accurate, complete, or fit for any purpose; or **that any data processed on a peer node will be kept confidential.** See Sections 5, 10, and 11.

---

## 3. Eligibility

You must be **at least 18 years old** (or the age of majority in your place of residence, if higher) to use the Service. By using the Service you represent and warrant that you meet this age requirement and have the legal capacity to enter into these Terms. An **age gate** and a **cookie banner** are presented before use; see the [Cookie Policy](cookie-policy.md).

**Pseudonymous participation (v1).** In the current version, participation is **pseudonymous**. A peer is identified solely by a locally generated **ed25519 keypair**, and the peer's public key serves as its peer identifier. In v1:

- there are **no accounts, usernames, real names, email addresses, passwords, profiles, or identity verification**; and
- we perform **no "Know Your Customer" (KYC)** or other identity or age verification beyond the self-attested age gate.

Because identity is a local keypair, **you are solely responsible for safeguarding your private key.** Loss of your key means loss of access to any contribution ratio associated with it, and we cannot recover, reset, reassign, or restore keys or ratio.

> **(Planned, v2)** Optional or mandatory **accounts, identity verification, and/or KYC** may be introduced, for example to support paid Credits, payouts, or compliance obligations. Such features are **not** offered today. Any future verification will be governed by updated terms and the [Privacy Policy](privacy-policy.md).

---

## 4. Access & Barter Credits

### 4.1 v1 — Barter (contribution ratio); no money

In the current version, **there is no money in the Service.** Access to inference is **earned by serving**, not purchased. Your ability to request inference is governed by a **contribution ratio** (broadly, an upload/download or served/consumed ratio) tracked by the coordinator's ledger.

The contribution ratio:

- is **non-transferable** and **revocable**;
- is **not** a currency, money, e-money, security, financial instrument, investment, commodity, or store of value;
- is **not** redeemable for cash and **has no cash or monetary value**;
- confers no ownership, equity, dividend, interest, or profit-sharing right; and
- may be adjusted, reset, suspended, or revoked to enforce these Terms, the [AUP](acceptable-use-policy.md), or the integrity of the network, or for technical reasons, in each case as described in Section 13.

You should not acquire, hold, or attempt to trade the contribution ratio in expectation of any economic return. It exists solely as an anti-freeloading, resource-allocation mechanism.

### 4.2 v2 — Purchased Credits (planned)

> **(Planned, v2)** We may in the future offer **purchased "Credits,"** provider **payouts**, a **platform fee disclosed in your dashboard before you serve**, and associated **tax reporting** and **KYC**. These features are **not** available today and are described here only for transparency.

> *Counsel note (retain):* Introducing paid, and especially **tradeable, transferable, or blockchain-based**, credits or payouts may trigger **securities, commodities, money-transmission / money-services-business, e-money, payments, consumer-credit, tax-reporting, and EU MiCA** (and comparable) regimes across multiple jurisdictions. **Do not enable paid Credits, transferability, payouts, or any on-chain representation without specific legal advice** and any required licensing/registration. Keep Credits strictly non-transferable and closed-loop absent such advice.

---

## 5. Requester Responsibilities

If you submit requests for inference ("**Requester**"), you are solely responsible for the data you submit and the use you make of any output. You represent, warrant, and agree that:

- **Lawfulness and rights.** You have all rights, licenses, permissions, and consents necessary to submit your input data and to have it processed by a Provider, and your submission and use of outputs comply with all applicable laws.
- **Providers see plaintext.** You understand that **v1 has NO technical mechanism preventing a Provider from reading, logging, or retaining your prompts or the resulting outputs.** The prohibition on such conduct is **contractual only** (imposed on Providers via the [Provider Agreement](provider-agreement.md) and [AUP](acceptable-use-policy.md)); confidentiality-protecting mechanisms such as trusted execution environments (TEEs) or sandboxing are **roadmap items and are not in effect.** Accordingly:

  > **Prohibited data.** Because Providers process prompts in plaintext with no technical isolation in v1, you **MUST NOT** submit: **(a) other people's personal data**; **(b) special-category or otherwise sensitive personal data** (for example biometric, health, racial or ethnic origin, sexual, religious, or political data); or **(c) regulated data**, including protected health information (PHI subject to HIPAA), financial information subject to GLBA, and cardholder data subject to PCI DSS. You may submit **your own non-sensitive content only.** No compliant tier for prohibited data exists in v1.

- **Third-party rights.** You will not submit data that infringes intellectual property, violates privacy, or breaches confidentiality or contractual obligations owed to others.
- **No reliance on outputs.** AI outputs may be **inaccurate, incomplete, biased, offensive, or otherwise unreliable.** You must independently verify outputs before relying on them. **Do not rely on outputs for medical, legal, financial, safety-critical, or other high-stakes decisions.** Outputs are not professional advice.
- **Peer IP exposure.** You understand that participating in the peer-to-peer network **exposes your network address (including your IP address) to peers** you connect with, and that you will likewise learn peers' IP addresses. See Section 6 of the [Privacy Policy](privacy-policy.md).

---

## 6. Intellectual Property

- **Your inputs and outputs.** As between you and us, **you retain all rights you hold in the inputs you submit and, to the extent permitted by applicable law and the applicable model license, in the outputs generated for you.** We claim no ownership of your inputs or outputs.
- **License to operate the Service.** You grant us and the relevant Provider a **limited, non-exclusive, worldwide, royalty-free license to receive, route, transmit, and process** your inputs and outputs **solely to provide the Service** (for us, on a metadata-only basis as described in Section 2; for the Provider, to perform the requested inference). This license ends when the relevant processing is complete, except for metadata retained per the [Privacy Policy](privacy-policy.md).
- **Model licenses are your responsibility.** Models served on the network are subject to their own licenses and use restrictions. **You are responsible for ensuring your use of any model and its outputs complies with that model's license and applicable law**, whether you are requesting inference or serving a model.
- **Platform software.** The p2ptokens software is made available as **open source under its applicable open-source license**; your rights in the software are governed by that license. Our **names, logos, and trademarks** are reserved, and nothing in these Terms or the open-source license grants you any right to use our marks except as permitted by applicable trademark law or a separate written agreement.

---

## 7. Acceptable Use

Your use of the Service is subject to the **[Acceptable Use Policy](acceptable-use-policy.md)**, which prohibits, among other things, unlawful, infringing, abusive, and harmful uses, and abuse of the network or contribution mechanism. You must comply with the AUP at all times.

**We monitor metadata, not content.** Because the coordinator is content-blind (Section 2), our enforcement is based on **metadata and reported conduct**, not on inspection of your prompts or outputs. This means violations may go undetected by us; it does not excuse non-compliance, and we may act on reports, patterns in metadata, or legal process.

---

## 8. Export Controls & Sanctions

You must comply with all applicable **export control, sanctions, and trade laws**, including those of the United States, the European Union, the United Kingdom, and any other applicable jurisdiction. You represent that you are not located in, ordinarily resident in, or acting on behalf of a person in an embargoed or comprehensively sanctioned jurisdiction, and that you are not a **denied, blocked, or sanctioned party**. You will not use the Service, or permit its use, in violation of such laws, including for prohibited end-uses or by prohibited end-users. Before enabling payments, the Operator will apply **IP-based geo-screening of embargoed jurisdictions**.

> *Counsel note (retain):* Export/sanctions rules applicable to **AI models, model weights, and compute** are **evolving rapidly** (e.g., US BIS controls on advanced computing, model-weight and diffusion rules; EU/UK measures). Reassess classification and screening obligations periodically, including whether serving or accessing particular models triggers controls.

---

## 9. DMCA / Copyright Complaints

We respect intellectual property rights and respond to notices of alleged infringement under the US Digital Millennium Copyright Act (DMCA) and comparable laws, to the extent applicable to our role as an intermediary.

If you believe content or conduct on the Service infringes your copyright, send a notice containing the information required by 17 U.S.C. § 512(c)(3) to our designated agent. The Operator will **register a DMCA agent with the U.S. Copyright Office** and keep the registration current; until updated here, direct all notices to:

- **Email:** legal@p2ptokens.com

Given the content-blind, peer-to-peer architecture, "removal" means the access restrictions available to us: we may **disable coordinator matching for, restrict network access to, and/or adjust or revoke the contribution ratio of** the peers associated with allegedly infringing material, and we terminate the access of **repeat infringers** in appropriate circumstances. Counter-notices may be submitted to the same address.

---

## 10. Disclaimers

THE SERVICE, THE COORDINATOR, AND ALL INFERENCE, OUTPUTS, AND CONTENT ARE PROVIDED **"AS IS"** AND **"AS AVAILABLE,"** WITH ALL FAULTS AND WITHOUT WARRANTIES OF ANY KIND. TO THE MAXIMUM EXTENT PERMITTED BY APPLICABLE LAW, [LEGAL ENTITY NAME] AND ITS AFFILIATES, OFFICERS, EMPLOYEES, AND SUPPLIERS **DISCLAIM ALL WARRANTIES, EXPRESS, IMPLIED, STATUTORY, OR OTHERWISE**, INCLUDING ANY IMPLIED WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, TITLE, AND NON-INFRINGEMENT, AND ANY WARRANTIES ARISING FROM COURSE OF DEALING OR USAGE OF TRADE.

WITHOUT LIMITING THE FOREGOING, WE EXPRESSLY DISCLAIM ANY WARRANTY OR REPRESENTATION:

- AS TO THE **CONDUCT, IDENTITY, TRUSTWORTHINESS, OR PERFORMANCE OF ANY PROVIDER OR OTHER PEER**, INCLUDING WHETHER A PROVIDER READS, LOGS, RETAINS, OR MISUSES YOUR PROMPTS OR OUTPUTS;
- AS TO THE **CONFIDENTIALITY, SECURITY, OR NON-DISCLOSURE OF ANY DATA PROCESSED ON, TRANSMITTED TO, OR STORED BY ANY PEER NODE** (V1 PROVIDES NO TECHNICAL CONFIDENTIALITY GUARANTEE ON PEER NODES — SEE SECTION 5); AND
- AS TO THE **ACCURACY, RELIABILITY, COMPLETENESS, TIMELINESS, OR FITNESS OF ANY AI OUTPUT.**

YOU ASSUME ALL RISK ARISING FROM YOUR USE OF THE SERVICE, YOUR SUBMISSION OF DATA TO PEERS, AND YOUR RELIANCE ON ANY OUTPUT.

**Jurisdictional savings clause.** Some jurisdictions do not allow the exclusion of certain warranties or the limitation of certain statutory or consumer rights. To the extent such law applies to you, the above disclaimers apply only to the fullest extent permitted, and nothing in these Terms limits any right that cannot lawfully be limited.

---

## 11. Limitation of Liability

TO THE MAXIMUM EXTENT PERMITTED BY APPLICABLE LAW:

**(a) No indirect damages.** [LEGAL ENTITY NAME] AND ITS AFFILIATES, OFFICERS, EMPLOYEES, AND SUPPLIERS WILL **NOT BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, CONSEQUENTIAL, EXEMPLARY, OR PUNITIVE DAMAGES**, OR FOR ANY **LOSS OF PROFITS, REVENUE, DATA, GOODWILL, OR BUSINESS**, ARISING OUT OF OR RELATING TO THE SERVICE OR THESE TERMS, WHETHER IN CONTRACT, TORT (INCLUDING NEGLIGENCE), STRICT LIABILITY, OR OTHERWISE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGES.

**(b) Aggregate cap.** OUR TOTAL AGGREGATE LIABILITY ARISING OUT OF OR RELATING TO THE SERVICE OR THESE TERMS WILL NOT EXCEED THE **GREATER OF (i) THE TOTAL AMOUNTS YOU PAID TO US, IF ANY, IN THE TWELVE (12) MONTHS BEFORE THE EVENT GIVING RISE TO THE LIABILITY, OR (ii) ONE HUNDRED U.S. DOLLARS (USD $100).** BECAUSE THE V1 BARTER MODEL INVOLVES NO PAYMENTS TO US, THIS CAP EFFECTIVELY **FLOORS AT USD $100** IN V1.

**(c) Statutory carve-outs.** Nothing in these Terms excludes or limits liability that cannot be excluded or limited under applicable law, including liability for **fraud or fraudulent misrepresentation, gross negligence, willful misconduct, death or personal injury caused by negligence,** or any **non-excludable consumer rights**. The exclusions and limitations above apply only to the extent permitted by the law applicable to you.

---

## 12. Indemnification

To the maximum extent permitted by applicable law, you will **defend, indemnify, and hold harmless** [Legal Entity Name] and its affiliates, officers, directors, employees, and agents from and against any claims, demands, liabilities, damages, losses, and expenses (including reasonable legal fees) arising out of or relating to: (a) your use of the Service; (b) the data you submit or the outputs you use, including any claim that they infringe or violate a third party's rights or any law; (c) your conduct as a Requester or Provider, including any breach of these Terms, the [AUP](acceptable-use-policy.md), or (if applicable) the [Provider Agreement](provider-agreement.md); (d) your use of third-party models, endpoints, or credentials; and (e) your violation of any applicable law. We may assume the exclusive defense and control of any matter subject to indemnification, at your expense, and you will cooperate with us.

This indemnity **does not apply to the extent it is prohibited or limited by mandatory consumer-protection law applicable to you**; where you use the Service as a consumer, it is limited to the maximum extent such law permits.

---

## 13. Term, Suspension & Termination

**Term.** These Terms apply for as long as you use the Service.

**Your right to stop.** You may stop using the Service at any time. Because v1 has no accounts, "closing" simply means ceasing to use the client and network; you may also discontinue use of your keypair.

**Suspension or termination for cause.** We may **suspend, restrict, or terminate** your access, and/or **adjust, freeze, reset, or revoke your contribution ratio (or, in v2, Credits)**, at any time, with or without notice, if we reasonably believe you have violated these Terms, the [AUP](acceptable-use-policy.md), or the [Provider Agreement](provider-agreement.md); if required by law or legal process; to protect the Service, other users, or third parties; or to address security, fraud, or network-integrity risks.

**Effect on unused ratio.** The v1 contribution ratio has **no cash or monetary value** (Section 4.1); upon termination it simply lapses and is not converted to any refund.

> **(Planned, v2)** For purchased **Credits**, on termination any **unused Credits will be refunded or rolled over — they are not forfeited**. This avoids consumer-law concerns with stored-value forfeiture. Termination for cause does not entitle us to withhold the value of unused paid Credits except to the extent expressly permitted by applicable law.

---

## 14. Dispute Resolution & Governing Law

**Governing law.** These Terms are governed by the **laws of the jurisdiction in which the Operator is established, excluding its conflict-of-laws rules**, except where **mandatory consumer-protection or other non-waivable laws** of your place of residence apply.

**Arbitration and class-action waiver.** Except as provided below, any dispute arising out of or relating to these Terms or the Service will be resolved by **final and binding individual arbitration** before a recognized arbitration body in the jurisdiction in which the Operator is established, under that body's rules, with the seat of arbitration in that same jurisdiction. Disputes will be resolved **individually**, and **NOT** in a class, collective, consolidated, or representative proceeding. Each party **WAIVES any right to a jury trial and to participate in a class action**, in each case to the extent permitted by law.

**Small-claims exception.** Notwithstanding the foregoing, either party may bring an individual claim in a **small-claims court** with jurisdiction, if the claim qualifies and remains in that court.

**Consumer carve-out.** If you are a **consumer** — including a consumer resident in the **European Union or the United Kingdom**, or in another jurisdiction with equivalent protections — the arbitration agreement and class-action waiver above **do not deprive you of any mandatory rights** under the law of your place of residence. To the extent those provisions conflict with mandatory consumer law, they **do not apply to you**: you retain all **mandatory statutory rights**, may bring claims in, and are not deprived of access to, the **courts of your place of residence**, and may use any available alternative or online dispute resolution mechanism.

---

## 15. General

- **Entire agreement.** These Terms, together with the incorporated [Privacy Policy](privacy-policy.md), [Cookie Policy](cookie-policy.md), [Acceptable Use Policy](acceptable-use-policy.md), and (for Providers) the [Provider Agreement](provider-agreement.md), constitute the entire agreement between you and us regarding the Service and supersede all prior understandings on that subject.
- **Severability.** If any provision is held unenforceable, it will be limited or severed to the minimum extent necessary, and the remaining provisions will remain in full force.
- **No waiver.** Our failure to enforce any provision is not a waiver of it.
- **Assignment.** You may not assign these Terms without our consent; we may assign them in connection with a merger, acquisition, or sale of assets.
- **Changes to these Terms.** We may modify these Terms from time to time. For material changes, we will provide at least **30 days' notice** by reasonable means before they take effect, except where a shorter period is required by law or to address security or legal risk. Notice will be given through our standing mechanisms — an **in-client banner, release notes, and a posting on p2ptokens.com** — and we retain a record of the notice. Your continued use after the changes take effect constitutes acceptance. If you do not agree, stop using the Service.

---

## Contact

Questions about these Terms may be sent to:

- **Email:** legal@p2ptokens.com
- **Operator:** [Legal Entity Name], [registered address]
- **Copyright / DMCA agent:** notices to legal@p2ptokens.com (agent to be registered with the U.S. Copyright Office)
- **Privacy:** privacy@p2ptokens.com
- **Security:** security@p2ptokens.com

For privacy matters, see the [Privacy Policy](privacy-policy.md). For acceptable-use questions, see the [Acceptable Use Policy](acceptable-use-policy.md). If you serve inference as a Provider, review the [Provider Agreement](provider-agreement.md).
