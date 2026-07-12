# Exhibit A — Data Processing Addendum (DPA)

**Between:** [Legal Entity Name] ("Operator") and the Provider ("Processor")
Incorporated into the [Provider Agreement](provider-agreement.md).

> **DRAFT — FOR ATTORNEY REVIEW.** GDPR Art. 28 / UK GDPR DPA. It intentionally states the v1 security limitations truthfully (see §6); do not remove or soften those. Prepared to reduce review burden; [Legal Entity Name] should still obtain a final legal review before publishing.

## 1. Roles and subject matter

Where a Requester's workload contains personal data, the Requester is the
controller (or itself a processor), the Operator's role is as described in the
[Privacy Policy](privacy-policy.md) §4.5, and the Provider
processes that personal data as a **sub-processor** solely to perform inference
requested through the network.

- **Nature/purpose:** executing AI inference on data transmitted to the
  Provider's node.
- **Duration:** the moment a job is processed; no persistent storage is
  authorized (§5).
- **Types of personal data:** whatever a Requester includes in a prompt or input
  (unknown to the Operator; the coordinator is content-blind).
- **Categories of data subjects:** any persons referenced in Requester inputs.

## 2. Processing only on instructions

The Provider processes workload personal data only to execute the job as directed
by the node software, and not for any other purpose. The Provider must not process
it for its own purposes, model training, profiling, or resale.

## 3. Confidentiality

The Provider ensures that anyone with access to the data is bound by
confidentiality.

## 4. No inspection, logging, or retention

The Provider must not read, log, copy, retain, transmit, or reverse-engineer
workload data beyond automatic in-memory processing required to run the job (see
[Provider Agreement](provider-agreement.md) §4).

## 5. Deletion / return

Because processing is transient and in-memory, workload personal data is not
persisted; the Provider must ensure it is not written to durable storage and is
discarded when the job completes.

## 6. Security (Art. 32) — honest statement of v1 measures

**In-transit:** all workload data is encrypted in transit (libp2p Noise).

**In-use — important:** the model runs on the data in **plaintext** in the
Provider's memory. **In v1 there is no technical isolation (no sandbox, no trusted
execution environment) enforced by the Operator.** Confidentiality in v1 rests on
the Provider's contractual obligations, reputation, and challenge-audits, **not**
on a technical barrier. Sandboxed/confidential execution is a roadmap item.

**Operator's Art. 32 position (v1).** The v1 technical and organizational measures
are: (a) encryption in transit (libp2p Noise); (b) content-blind coordination (the
Operator's coordinator does not see workload contents); (c) the contractual
no-inspection/no-logging/no-retention controls of Provider Agreement §4; and
(d) reputation and random challenge-audits. The Operator acknowledges that there is
**no in-use technical isolation in v1**. As the compensating measure that makes
these controls appropriate to the risk, the Operator **restricts the data permitted
on the network**: Requesters are barred (ToS/AUP) from submitting personal,
special-category, or otherwise regulated data. Where any restricted EU/UK personal
data is nonetheless in scope, the parties additionally rely on the transfer
mechanisms in §11.

## 7. Sub-processing

The Provider must not engage further sub-processors for workload data.

## 8. Assistance to the controller

Taking into account the nature of processing, the Provider will assist with data
subject requests and with security, breach, and DPIA obligations to the extent it
is able (noting it does not retain data and cannot generally identify data
subjects).

## 9. Breach notification

The Provider must notify **security@p2ptokens.com within 24 hours** of becoming
aware of any actual or suspected unauthorized access to, or retention of, workload
data on its node.

## 10. Audits

The Provider will make available information necessary to demonstrate compliance
and will cooperate with reasonable audits, including the random challenge-audits and
integrity challenges described in Provider Agreement §4.5. Audits are limited to the
Provider's handling of workload data and node configuration, conducted on reasonable
notice (or without notice for challenge-audits) and no more than once per year absent
suspected breach or a legal/regulatory requirement.

## 11. International transfers

The network is global; workload data may be processed on nodes worldwide. For
restricted EU/UK personal data, the parties rely on the applicable **Standard
Contractual Clauses (EU Commission Implementing Decision (EU) 2021/914)** and, for
UK transfers, the **UK International Data Transfer Addendum**, which are incorporated
by reference and apply to such transfers, plus the supplementary measures described
in the Privacy Policy and §6.

## 12. Precedence

If this DPA conflicts with the Provider Agreement on data-protection matters, this
DPA controls.
