# Acceptable Use Policy

> **DRAFT — FOR ATTORNEY REVIEW.** Prepared to reduce review burden; [Legal Entity Name] should still obtain a final legal review before publishing.

**Operator:** [Legal Entity Name] ("we," "us," or "our"), operator of the product **p2ptokens**.

**Effective date:** [Effective Date]
**Last updated:** [Last Updated]

This Acceptable Use Policy ("AUP") governs your use of the p2ptokens network instance operated by [Legal Entity Name]. It should be read together with our [Terms of Service](terms-of-service.md), [Privacy Policy](privacy-policy.md), and, for Providers, the [Provider Agreement](provider-agreement.md).

---

## 1. Purpose & Scope

p2ptokens is a peer-to-peer ("P2P") artificial-intelligence inference network. Participants act as **Requesters** (who submit inference jobs) and/or **Providers** (who serve those jobs from locally hosted models such as Ollama, or from a third-party inference endpoint they proxy using their own credentials). A **coordinator** matches peers and meters exchange, but is **content-blind**: it sees only metadata (peer identifiers, IP addresses, model names, token counts) and never the content of prompts or outputs. Identity on the network is a pseudonymous ed25519 keypair; in v1 there are no accounts and no identity verification (KYC). Access is governed by a barter ratio rather than money.

This AUP applies to **all users** of the instance operated by [Legal Entity Name], whether acting as a Requester, a Provider, or both. It is **incorporated by reference into, and forms part of, the [Terms of Service](terms-of-service.md)**. Capitalized terms not defined here have the meaning given in the Terms of Service. Violation of this AUP is a violation of the Terms of Service.

By accessing or using the network, you agree to comply with this AUP. If you do not agree, you must not use the network.

---

## 2. Prohibited Content & Conduct

You must not use the network — as a Requester, Provider, or otherwise — to create, submit, request, serve, transmit, store, or facilitate any of the following. You must not use the network to:

- **Violate any applicable law or regulation**, or to promote, facilitate, or enable any unlawful act.
- Produce, solicit, distribute, or process **child sexual abuse material (CSAM)** or any content that **sexualizes minors**, whether real, synthetic, or AI-generated.
- Support, promote, or facilitate **terrorism or violent extremism**, or incite or glorify violence.
- Develop, design, produce, or facilitate **weapons**, including **chemical, biological, radiological, or nuclear (CBRN)** weapons, or their means of delivery.
- Create, distribute, or facilitate **malware, ransomware, exploits, or other malicious code**, or the tooling to develop them.
- Engage in or facilitate **fraud, phishing, deceptive practices, spam, or bulk unsolicited messaging**.
- Create, request, or distribute **non-consensual intimate imagery** (including sexual deepfakes) or other content that violates a person's sexual privacy or dignity.
- Conduct **unlawful surveillance**, tracking, profiling, or other **violations of privacy**, or process personal data in violation of applicable privacy law.
- **Infringe intellectual property rights**, including copyrights, trademarks, patents, or trade secrets, or misappropriate the rights of others.
- Generate or distribute content intended to **harass, bully, threaten, defame, or degrade** any person or group.
- **Circumvent, defeat, or attempt to compromise the security** of any person, system, network, or account.

This list is illustrative, not exhaustive. We may treat other conduct that is harmful, abusive, or contrary to the spirit of this AUP as prohibited.

---

## 3. Network-Integrity Rules

To protect the availability, integrity, and fairness of the network for all participants, you must not:

- **Probe, scan, disrupt, overload, degrade, or attack** — including through denial-of-service (DoS) or distributed denial-of-service (DDoS) — the coordinator, relays, provider nodes, or any other network infrastructure or peer.
- Attempt to **access, extract, intercept, decrypt, or infer other users' job data**, prompts, outputs, or any content not intended for you.
- **Circumvent, forge, manipulate, or tamper with** the metering, co-receipt, ratio, or reputation systems, or otherwise misrepresent your contribution or consumption.
- **Disable, bypass, modify, or tamper with** any execution isolation, sandboxing, or other security mechanism, whether currently deployed or introduced in the future.
- **Impersonate** another peer, spoof or forge cryptographic keys or identifiers, or otherwise misrepresent your identity or affiliation.

---

## 4. Requester Obligations

If you submit inference jobs, you are solely responsible for the data you submit and the use you make of the results. Specifically:

- You are responsible for the **lawfulness of, and for having all necessary rights, licenses, consents, and authority over,** any data, prompt, file, or other material you submit.
- **You must NOT submit other people's, sensitive, or regulated data.** Because **Providers process prompts in plaintext** and, in v1, there is **no technical isolation, sandboxing, or other mechanism preventing a Provider from logging or retaining them** (protection is contractual and reputational only), you **must not** submit:
  - **(a) other people's personal data** — any personal data relating to a person other than yourself;
  - **(b) special-category or otherwise sensitive personal data**, including **biometric identifiers**, health, racial or ethnic origin, sexual, religious, or political data; and
  - **(c) regulated data**, including **protected health information (PHI) subject to HIPAA**, **financial data subject to the GLBA**, and **payment card data subject to PCI DSS**.
  You may submit **your own non-sensitive content only.** Do not submit trade secrets or confidential information of yourself or third parties. No compliant tier for the prohibited data above exists in v1.
- You are responsible for **evaluating, validating, and using the outputs** you receive. AI outputs may be inaccurate, incomplete, or unsuitable for your purpose; you must not rely on them without independent verification, and you are responsible for any use, distribution, or downstream effect of those outputs.

See the [Privacy Policy](privacy-policy.md) for more on how the network handles data and metadata.

---

## 5. Provider Obligations

If you serve inference jobs, you are entrusted with other users' workloads and must protect them. Specifically:

- You must **not inspect, log, read, retain, store, copy, exfiltrate, or otherwise use** the workload data (including prompts and outputs) that passes through your node, except as strictly necessary to serve the job in real time. These obligations are **contractual**; see the [Provider Agreement](provider-agreement.md).
- If you serve jobs by proxying a **third-party model or endpoint**, you must have the **legal right** to use that model, endpoint, service, and the **credentials** employed, and to make it available to other users through the network. You **bear the full risk** that proxying may violate the third party's terms of service, license, or usage policies, and you are solely responsible for any resulting liability.
- You must run **only official, unmodified node software** as distributed by [Legal Entity Name]. You must not alter, patch, instrument, or replace the node software in any way that affects security, metering, isolation, or workload confidentiality.

---

## 6. Export Controls & Sanctions

You may not access or use the network, and may not provide or receive services through it:

- from, or on behalf of any person or entity located in, any **embargoed or comprehensively sanctioned jurisdiction**; or
- if you are a **sanctioned, restricted, or denied party**, or are acting on behalf of one, under the laws of the United States or any other applicable jurisdiction.

You are responsible for complying with all applicable **export-control, sanctions, and trade laws**, and you represent that your use of the network does not violate any of them. Before enabling payments, the Operator will apply **IP-based geo-screening of embargoed jurisdictions**.

---

## 7. Enforcement

The coordinator is **content-blind** and **cannot see the content** of prompts or outputs. Our monitoring is therefore limited to **metadata** (such as peer identifiers, IP addresses, model names, and token counts). We do not, and technically cannot, review the substance of your jobs at the coordinator level.

Where we identify, or receive credible reports of, conduct that violates this AUP or applicable law, we may take any action we consider appropriate, including:

- **throttling** or rate-limiting;
- **ratio and reputation penalties**;
- **rejecting jobs** or refusing to match peers;
- **suspending or terminating** access to the instance we operate;
- **referring matters to law enforcement or other authorities**; and
- **preserving and disclosing** metadata and other information as permitted or required by law.

We may act in response to **abuse reports, third-party complaints, subpoenas, court orders, and other legal requests**. We are not obligated to monitor, and our decision not to act in any instance does not waive our right to act in others.

---

## 8. Reporting Abuse

If you become aware of a violation of this AUP, unlawful content, or a security concern, please report it promptly:

- **General / legal / abuse:** legal@p2ptokens.com
- **Security vulnerabilities and incidents:** security@p2ptokens.com

Please include as much detail as you can (relevant peer identifiers, timestamps, and a description). Do not include unlawful content itself in your report.

---

## 9. Self-Hosted Instances

The p2ptokens software is **open source and self-hostable**. This AUP governs **only the network instance operated by [Legal Entity Name]**. Instances deployed, operated, or federated by other parties are **independently operated and outside our control**. We are not responsible for the conduct, policies, availability, or security of any instance we do not operate, and this AUP does not apply to them.

---

## 10. Changes to This Policy

We may update this AUP from time to time. When we do, we will revise the effective date above and make the updated version available through the usual channels. Material changes may be communicated by additional means where practicable. Your continued use of the network after an updated AUP takes effect constitutes your acceptance of the changes. If you do not agree, you must stop using the network.

---

*This document is provided for the instance operated by [Legal Entity Name]. Questions about this AUP may be directed to legal@p2ptokens.com.*
