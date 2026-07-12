# Exhibit B — Minimum Hardware & Security Specifications

Incorporated into the [Provider Agreement](provider-agreement.md) §3.

> **DRAFT.** Operational requirements; the values below are current minimums and the Operator may update them on reasonable notice. Prepared to reduce review burden; [Legal Entity Name] should still obtain a final legal review before publishing.

## 1. Hardware (minimums)

- **GPU/compute:** a GPU with ≥ 8 GB VRAM for small models, or ≥ 24 GB for
  larger models — or a CPU capable of running the models you advertise at acceptable
  latency.
- **RAM:** ≥ 16 GB system RAM.
- **Disk:** enough for the model weights you serve.
- **Network:** a stable connection with ≥ 20 Mbps up; a public IP or
  working NAT traversal (the client supports relay/DCUtR).

## 2. Software

- Run **only official, unmodified** p2ptokens node software.
- Keep the node client, GPU drivers, and OS security-patched and up to date.
- Run a supported model backend (e.g., Ollama) or a third-party endpoint you are
  authorized to proxy.

## 3. Security obligations

- Maintain reasonable physical and network security over the node host.
- **Full-disk encryption is strongly recommended** (workloads are processed in
  plaintext in memory; disk encryption reduces at-rest exposure of any incidental
  data).
- Do not run the node on hardware, networks, or electricity you are not authorized
  to use (e.g., employer/institutional resources without permission).
- Do not co-locate the node with software that inspects or captures its memory or
  traffic.
- Apply your own defensive measures against untrusted workloads; **v1 provides no
  execution sandbox** (see Provider Agreement §4.6, §8.3).

## 4. Prohibited environments

- Do not operate a node from an embargoed jurisdiction or while a sanctioned
  party (see Provider Agreement §6).
