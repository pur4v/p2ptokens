# Provider Agreement

> **DRAFT — FOR ATTORNEY REVIEW.** Prepared to reduce review burden; [Legal Entity Name] should still obtain a final legal review before publishing.

**Operator:** [Legal Entity Name] ("Operator", "we", "us"), operator of the product **"p2ptokens"**.

**Provider:** the person or entity that operates a p2ptokens node to contribute compute to the network, identified by an ed25519 public key ("you", "Provider").

**Effective date:** accepted electronically when you first run the node software, or on [Effective Date], whichever is earlier.

---

## Preliminary Notes

*This Agreement describes the p2ptokens network as it exists in version 1 ("v1"). Several features referenced below — cash payouts, platform fees, minimum thresholds, tax documentation and reporting, and sandboxed/confidential execution — are **not present in v1**. They are marked "(planned, v2)" or "roadmap" where they appear. Nothing in a "(planned, v2)" clause is a present commitment, a guarantee of future delivery, or an entitlement; those clauses take effect only if and when the corresponding feature launches.*

*Acceptance is by click-wrap or run-wrap: you accept this Agreement when you first run the node software, and that action binds you to it. Because v1 providers are pseudonymous ed25519 keypairs with no accounts or KYC (see Section 2 and Section 6), any mandatory consumer-protection and unfair-terms protections that apply to an individual acting as a Provider are preserved as required by applicable law and by the ToS; nothing in this Agreement waives protections that cannot lawfully be waived.*

---

## 1. Scope; Relationship to the Terms of Service

1.1 This Provider Agreement ("Agreement") governs your participation as a compute Provider on the p2ptokens network. It **supplements** and is incorporated into the [Terms of Service](./terms-of-service.md) ("ToS"). Capitalized terms not defined here have the meaning given in the ToS.

1.2 You are also bound by the [Acceptable Use Policy](./acceptable-use-policy.md) ("AUP") and the [Privacy Policy](./privacy-policy.md). The Privacy Policy and Exhibit A (Data Processing Addendum) govern personal data you may handle while processing workloads.

1.3 If this Agreement conflicts with the ToS on a Provider-specific matter, this Agreement controls to the extent of the conflict. On all other matters, the ToS controls.

1.4 **What a Provider does.** A Provider contributes inference compute in one of two ways:
   - **(a) Local models** — running models locally (e.g., via Ollama) on hardware you control; or
   - **(b) Proxied third-party endpoint** — relaying requests to a third-party inference endpoint using **your own credentials/API keys** (see Sections 3.6 and 6).

1.5 **How the network operates (v1).** Inference is peer-to-peer and **encrypted in transit**. The Operator's coordinator is **content-blind** and does not see workload contents. Because inference is peer-to-peer, **peers can see each other's IP addresses**. You acknowledge this network design and the privacy consequences described in Section 4 and the Privacy Policy.

---

## 2. Independent Contractor Status

2.1 It is the **Operator's position that you are an independent contractor**. Nothing in this Agreement creates an employment, agency, partnership, joint venture, or franchise relationship between you and the Operator. You are responsible for determining and complying with how your activity is classified under the law of your own jurisdiction.

2.2 You control the manner and means by which you provide compute, subject to the technical and conduct requirements of this Agreement, the ToS, and the AUP. You supply your own hardware, software environment, connectivity, and electricity.

2.3 You are not entitled to any employee benefits, and you are solely responsible for your own taxes, insurance, and regulatory obligations arising from your activity (see Section 5).

2.4 **Pseudonymity (v1).** Providers participate pseudonymously through ed25519 keypairs. **There are no accounts and no KYC in v1.** You are responsible for safeguarding your private key; control of the key is treated as control of the Provider identity. The Operator cannot recover, reset, or reassign your key.

---

## 3. Hardware and Node Requirements

3.1 **Official, unmodified software.** You must run the **official, unmodified** p2ptokens node client, obtained from an official distribution channel, and keep it reasonably up to date. You must not modify, patch, reverse-engineer for circumvention, fork-and-run against the production network, or otherwise alter the client in a way that changes its protocol behavior, its security controls, or its data-handling behavior. Running modified client software is a material breach.

3.2 **Open-source note.** The node software is open source. Section 3.1 does not restrict your rights under the applicable open-source license (including to study, modify, and redistribute the code); it restricts **connecting a modified client to the production p2ptokens network** and representing it as an official node.

3.3 **Minimum specifications.** Your node must meet the minimum hardware and security specifications in **Exhibit B**. The Operator may update Exhibit B on reasonable notice as the network evolves.

3.4 **Security of your environment.** You must maintain reasonable, current security controls on the host running the node, including OS and dependency patching, malware protection appropriate to the platform, secure storage of your private key and any third-party credentials, and restriction of administrative access. You are responsible for the security posture of the machine and network you connect.

3.5 **Right to the hardware and electricity.** You represent and warrant that you have the **legal right to use the hardware, facilities, network connection, and electricity** used to operate your node — including that such use does not violate any employer, landlord, institutional, university, data-center, cloud-provider, hosting, or ISP policy or agreement, and does not misappropriate another party's resources. Using resources you are not authorized to use is a material breach and is solely your risk and liability (see Section 6.4).

3.6 **Right to proxy a third-party backend.** If you operate in proxy mode (Section 1.4(b)), you represent and warrant that you have the **right to use the third-party endpoint and the credentials/API keys** you configure, and the right to relay p2ptokens network traffic through them. **Proxying a paid or restricted third-party endpoint may violate that third party's terms of service.** You accept that doing so is **entirely your own risk and liability**; the Operator does not provide, fund, or endorse any third-party endpoint and is not a party to your relationship with any third-party provider. See Sections 6 and 8.

---

## 4. Data-Handling Obligations — Core

*This Section is the most important part of this Agreement. Read it carefully.*

4.1 **What you can see.** When you process a workload, **you can see the workload data in plaintext** on your node. Inference input and output pass through your node's memory and software in clear form so the model can run.

4.2 **Critical acknowledgement — no technical enforcement in v1.**
> **In v1 there is NO technical mechanism — no sandbox and no trusted execution environment (TEE) — that prevents a Provider from reading, logging, or retaining workload data. The node software does not currently prevent this.**

You expressly acknowledge and agree that:
   - the confidentiality of workloads in v1 rests on **your contractual promises**, your good faith, and the network controls in Section 4.5 — **not** on any technical barrier that the Operator represents to exist;
   - the Operator makes **no representation** that the software technically prevents inspection, logging, or retention of workload data by a Provider; and
   - nothing in this Agreement, the ToS, the AUP, or any marketing should be read as a claim that such technical protection currently exists.

4.3 **Contractual no-inspection / no-logging / no-retention.** Notwithstanding your technical ability to do so, you **must not**:
   - **(a) inspect** workload contents beyond what is strictly and automatically required for the model to compute a response;
   - **(b) log, record, capture, cache beyond the ephemeral duration of processing, or copy** workload inputs, outputs, prompts, embeddings, or any derived data; or
   - **(c) retain** any workload data after the request is served, or transmit, sell, share, train on, or otherwise use it for any purpose other than serving that request.

   You must delete any workload data from memory and any transient buffers as soon as the request is complete.

4.4 **Breach and legal exposure.** Inspecting, logging, retaining, exfiltrating, or otherwise misusing workload data is a **material breach** of this Agreement. It may also violate **computer-misuse, data-protection, wiretap/interception, trade-secret, and other laws**, exposing you to **civil and criminal liability**. When technical protections described in Section 4.6 are later deployed, **circumventing, disabling, or tampering with them** is likewise a material breach and may independently violate such laws.

4.5 **Current controls (v1).** In v1 the controls on Provider data handling are:
   - **reputation** — misconduct that is detected reduces or eliminates your reputation and your ability to consume on the network; and
   - **random challenge-audits** — the Operator may issue crafted or canary workloads and other integrity challenges to detect inspection, logging, retention, tampering, or non-conforming behavior.

   You consent to such challenge-audits and agree not to attempt to detect, evade, or special-case them.

4.6 **Planned technical controls (roadmap).** Technical enforcement is on the roadmap and is **not present in v1**: **sandboxed execution** and **confidential compute / trusted execution environments (TEE)** are planned so that future versions can technically constrain access to workload data. These are goals, not commitments, and no timeline is promised. When deployed, you must run them and must not disable or circumvent them (Section 8.4).

4.7 **Prohibited data.** Requesters are **barred by the ToS and AUP from submitting personal, sensitive/special-category, or regulated data** to the network. This restriction does not lessen your duties: you must still treat **any** data you see while processing a workload as Confidential Information under this Section and Section 10, regardless of whether a Requester complied with that restriction.

4.8 **Personal data / sub-processor terms.** To the extent workload data nonetheless contains personal data, you act as a **sub-processor** with respect to that data, and your handling is governed by the Data Processing Addendum in **Exhibit A** and the [Privacy Policy](./privacy-policy.md). Exhibit A prevails over this Section on personal-data specifics.

4.9 **Breach notification — 24 hours.** You must notify the Operator at **security@p2ptokens.com within 24 hours** of becoming aware of any actual or suspected security incident, unauthorized access, loss, or exposure affecting workload data, your private key, your host, or your proxied credentials, and cooperate fully in investigation and remediation. Legal and compliance matters may also be directed to **legal@p2ptokens.com**.

---

## 5. Compensation and Taxes

5.1 **v1 — no money.** **There is no cash compensation in v1.** For contributed compute you earn a **barter ratio** and **reputation**, which entitle you to **consume** compute on the network. You do **not** earn cash, tokens redeemable for cash, or any monetary payout in v1. The barter ratio and reputation have no guaranteed exchange value and are not a security, deposit, or stored-value instrument.

5.2 **Cash payouts (planned, v2).** The following clauses take effect **only if and when paid payouts launch** and are **not operative in v1**:
   - **(a) Payouts (planned, v2).** Eligible providers may receive cash or monetary payouts for contributed compute per the schedule in **Exhibit C**.
   - **(b) Platform fee (planned, v2).** The Operator may deduct a platform fee as set out in Exhibit C.
   - **(c) Minimum threshold (planned, v2).** Payouts may be subject to a minimum accrual threshold before disbursement, per Exhibit C.
   - **(d) Tax documentation (planned, v2).** As a condition of receiving payouts, you may be required to provide valid tax documentation, including **IRS Form W-9** (US persons) or the applicable **Form W-8 series** (non-US persons), and to complete identity/KYC and sanctions screening.
   - **(e) Tax reporting (planned, v2).** The Operator may be required to report payments and/or withhold tax, including **US Form 1099** reporting and, in the EU, **DAC7** platform-operator reporting, and you agree to provide the information necessary for such reporting.

5.3 **Your tax responsibility.** In both v1 and v2, you are solely responsible for determining and satisfying your own tax obligations arising from your participation (including any tax consequences of barter/in-kind consideration under applicable law). The Operator does not provide tax advice.

5.4 **Operator tax collection and reporting (planned, v2).** When payouts launch, the Operator will collect the required tax documentation (W-9 for US persons; the applicable W-8 series for non-US persons) and will report and/or withhold where legally required, including US Form 1099 reporting and EU DAC7 platform-operator reporting. *(v2; to be finalized with a tax advisor before the first payout.)*

---

## 6. Compliance Representations

You represent, warrant, and covenant, on a continuing basis, that:

6.1 **Sanctions and export.** You are not located in, ordinarily resident in, organized under the laws of, or under the control of any person in a **comprehensively embargoed or sanctioned jurisdiction**, and you are not a **sanctioned or restricted party** (including on US OFAC SDN, EU, UK, or UN lists) or acting on behalf of one. You will comply with all applicable **sanctions and export-control laws** and will not use or make the node available in violation of them.

6.2 **Authority and no conflict.** You have the right and authority to enter into this Agreement, and your participation does not violate any other agreement or obligation binding on you — including any **employer, ISP, hosting/cloud-provider, university, or institutional** policy or agreement (see Section 3.5).

6.3 **Third-party endpoint rights.** If proxying (Section 3.6), your use of the third-party endpoint and credentials is authorized and does not violate that third party's terms; you bear all risk and liability arising from that use.

6.4 **Lawful resources.** You have the legal right to all hardware, facilities, connectivity, and electricity used, as set out in Section 3.5.

6.5 **Compliance with policies and law.** You will comply with this Agreement, the ToS, the AUP, and all applicable laws in operating your node.

---

## 7. Availability; No Guarantee of Work

7.1 **No guarantee of work.** The Operator does **not** guarantee that any workloads will be routed to your node, that any particular volume, ratio, reputation gain, or (in v2) earnings will result, or that the network will remain available. Work allocation depends on demand, routing, your reputation, your availability, and factors outside the Operator's control.

7.2 **No exclusivity; your discretion.** You may run, pause, or stop your node at any time. Nothing requires you to accept any minimum level of work, and nothing here restricts you from providing compute to others (subject to Section 6.2).

7.3 **No earnings promise; substantiation rule.** Any figures, examples, calculators, or illustrations regarding ratio, reputation, or (v2) earnings are **illustrative only, not guarantees**, and actual results will vary and may be zero. Any earnings, income, or return figure the Operator publishes in marketing, dashboards, or in-product estimators will be **substantiated and clearly labeled as illustrative**, will avoid atypical or cherry-picked results, and **no earnings are guaranteed**.

---

## 8. Risk Allocation and Liability

8.1 **Hardware wear and operating costs.** You accept all risk of **hardware wear, degradation, failure, and increased electricity and cooling costs** arising from operating your node. These are your costs, not the Operator's.

8.2 **Malicious or harmful workloads.** Workloads originate from third parties and pass through your node in plaintext. The Operator does not pre-screen workload contents (the coordinator is content-blind). You may be exposed to workloads that are offensive, unlawful in your jurisdiction, or crafted to probe or attack your node.

8.3 **Residual risk accepted (no sandbox in v1).** Because **sandboxed/confidential execution is PLANNED and not present in v1** (Section 4.6), workloads execute without a technical isolation barrier that the Operator represents to exist. **You accept the residual risk** of running third-party workloads on your own hardware in this configuration, including any impact on your system or data. You are responsible for your own defensive measures (Section 3.4).

8.4 **Do not disable future protections.** When sandbox/TEE or other protective controls are deployed, you must run them as intended and **must not disable, weaken, or circumvent** them. Doing so is a material breach, voids the protections of this Section for the affected conduct, and may create the legal exposure described in Section 4.4.

8.5 **Indemnity.** You will **indemnify, defend, and hold harmless** the Operator and its affiliates, officers, and personnel from and against any third-party claims, losses, liabilities, damages, penalties, and reasonable costs (including legal fees) arising out of or relating to: (a) your breach of this Agreement, the ToS, or the AUP; (b) your inspection, logging, retention, or misuse of workload data; (c) your proxying of a third-party endpoint or use of any credentials (Section 3.6); (d) your unauthorized use of hardware, facilities, or electricity (Section 3.5); (e) your violation of law, including sanctions/export, data-protection, and computer-misuse laws; or (f) tax obligations that are yours under Section 5.

8.6 **Limitation of liability.** To the maximum extent permitted by law, the Operator will not be liable for indirect, incidental, special, consequential, exemplary, or punitive damages, or for lost profits, revenue, data, or goodwill. **The Operator's total aggregate liability arising out of or relating to this Agreement is capped at the total amounts actually paid by the Operator to you in cash in the 3 months preceding the event giving rise to the claim** — which, **under the v1 barter model in which no cash is paid, is USD $0**. This cap will scale with actual cash payouts if and when v2 payouts launch. The Provider's own liability to the Operator is capped on the same basis (amounts paid to the Provider in the 3 months before the claim; USD $0 under the v1 barter model), subject to the same statutory carve-outs in Section 8.7.

8.7 **Statutory carve-outs.** Nothing in this Section limits or excludes liability that cannot lawfully be limited or excluded, including (as applicable) liability for death or personal injury caused by negligence, fraud or fraudulent misrepresentation, or any other liability that applicable law prohibits limiting. Where a jurisdiction does not permit certain exclusions or the stated cap, they apply to the maximum extent permitted.

---

## 9. Suspension and Termination

9.1 **At-will termination.** Either party may terminate this Agreement for any or no reason on **7 days'** notice. You may terminate by ceasing to operate your node.

9.2 **Immediate suspension or termination for cause.** The Operator may suspend or terminate your participation **immediately** and without notice for: material breach (including Sections 3, 4, 6, and 8.4); running modified client software; suspected inspection/logging/retention/exfiltration of workload data; failed challenge-audits; suspected sanctions/export or other legal violations; or conduct that threatens the network or other users.

9.3 **Effect of termination.** On termination you must stop operating the node and delete any workload data still in your possession (which, under Section 4, you should not retain in any event). Sections 4 (surviving confidentiality/data duties), 5.3, 6, 8, 10, and 11 survive.

9.4 **Accrued earnings (v2).** **If and when cash payouts exist**, on termination you remain entitled to **accrued, verified earnings** for conforming compute already provided, subject to Exhibit C (threshold, fee, tax documentation) and to forfeiture for termination arising from your material breach or fraud, to the extent permitted by law. **In v1 there are no cash earnings and nothing is payable on termination.**

---

## 10. Confidentiality

10.1 **Open-source scope.** Because the p2ptokens software is **open source**, publicly available code, documentation, and protocol details are **not** confidential. "Confidential Information" is limited to **non-public** information the Operator discloses to you or that you learn through participation, such as non-public **operational, architecture, pricing, and security** information (including non-public details of challenge-audits, canaries, anti-abuse, routing internals, and unreleased roadmap specifics).

10.2 **Obligations.** You will keep Confidential Information confidential, use it only to participate as a Provider, and not disclose it, for the term and for 2 years after (indefinitely for trade secrets and security information). This is separate from, and additional to, the workload-data duties in Section 4, which apply regardless of whether workload data is "confidential."

10.3 **Exclusions.** The usual exclusions apply (information that is or becomes public without your breach, was rightfully known to you without duty, is independently developed, or is rightfully received from a third party), as does disclosure required by law with reasonable prior notice where lawful.

---

## 11. General

11.1 **Governing law and disputes.** The **governing law and dispute-resolution provisions of the [Terms of Service](./terms-of-service.md) apply to this Agreement** and are incorporated by reference, including any arbitration, venue, and class-action provisions stated there.

11.2 **Entire agreement.** This Agreement, together with the ToS, AUP, Privacy Policy, and the Exhibits, is the **entire agreement** between the parties on its subject matter and supersedes prior understandings on that subject matter.

11.3 **Amendments.** The Operator may update this Agreement and the Exhibits on reasonable notice (including in-product or via an official channel). Continued operation of your node after the effective date of an update constitutes acceptance. Material changes affecting v2 compensation will be identified as such.

11.4 **Severability; waiver; assignment.** If any provision is held unenforceable, the rest remains in effect. No waiver is implied by delay. You may not assign this Agreement without the Operator's consent; the Operator may assign it to an affiliate or successor.

11.5 **Notices.** Operator notices to you may be given through the software or an official channel. Your notices to the Operator go to **legal@p2ptokens.com** (security incidents: **security@p2ptokens.com**, within 24 hours per Section 4.9).

---

## Exhibit A — Data Processing Addendum (GDPR Art. 28)

*The full Data Processing Addendum is provided as a separate document, [Exhibit A — DPA](./exhibit-a-dpa.md), and is incorporated here by reference.*

Exhibit A sets out the data-processing terms that apply where workload data contains personal data, with the Requester/customer as controller (or processor) and the Provider as **sub-processor**, structured to satisfy **GDPR Article 28** (and analogous laws). It addresses, at minimum:

- subject-matter, duration, nature, and purpose of processing; categories of data subjects and personal data;
- **processing only on documented instructions** (limited to serving the request; no inspection/logging/retention per Section 4);
- confidentiality of authorized persons;
- **security measures** appropriate to the risk, with explicit acknowledgement that **v1 lacks technical isolation (no sandbox/TEE)** and relies on contractual controls, reputation, and challenge-audits, with technical measures on the roadmap;
- sub-processing terms and prior authorization;
- assistance with data-subject rights and with security, breach-notification (aligned to the **24-hour** notice in Section 4.9), DPIA, and consultation obligations;
- **deletion/return** of personal data at end of processing;
- audit and information rights, including challenge-audits;
- **international transfer** mechanisms (Standard Contractual Clauses / UK Addendum for restricted transfers, noting peers see each other's IP addresses and inference is cross-border P2P); and
- allocation of liability consistent with Section 8.

Consistent with the honest v1 posture in Section 4, Exhibit A discloses that v1 security measures are encryption in transit plus content-blind coordination, contractual controls, reputation, and challenge-audits, with **no in-use technical isolation (no sandbox/TEE)**. The Operator's stated compensating measure under Art. 32 is to **restrict the data permitted on the network** — no personal, sensitive/special-category, or regulated data (see the ToS and AUP) — so that the organizational/contractual controls are appropriate to the data actually allowed.

---

## Exhibit B — Minimum Hardware and Security Specifications

*The full specifications are provided as a separate document, [Exhibit B — Minimum Hardware & Security Specifications](./exhibit-b-hardware-security-specs.md), and are incorporated here by reference. The Operator may update them on reasonable notice (Section 3.3). The values below are current operational minimums, stated as examples.*

**Hardware / environment:**
- GPU / accelerator: a GPU with at least 8 GB VRAM for small models, or at least 24 GB for larger models; alternatively a CPU able to run the models you advertise at acceptable latency
- CPU / system RAM: at least 16 GB system RAM
- Disk: enough free space for the model weights you serve
- Network: a stable connection with at least 20 Mbps upstream; a public IP or working NAT traversal (the client supports relay/DCUtR)
- Supported model runtime: Ollama (current supported version) or a third-party endpoint you are authorized to proxy

**Software:**
- Official, unmodified p2ptokens node client, kept updated (Section 3.1)

**Security (minimum):**
- OS and dependency patching current
- Secure storage of the ed25519 private key and any proxied third-party credentials
- Restricted administrative/network access to the host
- Malware protection appropriate to the platform
- Compliance with the incident-notification duty in Section 4.9 (security@p2ptokens.com within 24 hours)

---

## Exhibit C — Payout Schedule and Fee Table (v2)

> **(Planned, v2 — NOT operative in v1.)** In v1 there are no cash payouts; providers earn only a barter ratio and reputation (Section 5.1). This Exhibit takes effect only if and when paid payouts launch.

The full schedule is provided as a separate document, [Exhibit C — Payout Schedule and Fee Table](./exhibit-c-fees-payouts.md), and is incorporated here by reference. When payouts launch, it will cover:
- **Payout rates / conversion** from contributed compute to monetary value, as published in the Provider dashboard when payouts launch (v2)
- **Platform fee:** a platform fee disclosed in the Provider dashboard
- **Minimum payout threshold:** as published in the Provider dashboard when payouts launch (v2)
- **Payout cadence and method:** as published in the Provider dashboard when payouts launch (v2)
- **Tax documentation prerequisites:** W-9 (US) / W-8 series (non-US); KYC and sanctions screening (Section 5.2(d))
- **Reporting:** US 1099; EU DAC7 (Section 5.2(e))
- **Withholding:** backup/cross-border withholding as applicable
- **Currency, FX, and chargeback/clawback terms**

*(v2; to be finalized with a tax advisor before the first payout.)*

---

*End of Provider Agreement.*
