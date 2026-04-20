# Drift Protocol Forensic Investigation 🔍

This repository contains a comprehensive technical forensic audit and Proof-of-Concept (PoC) evidence regarding the security architecture failures of Drift Protocol, leading to the **$285M exploit** on April 1, 2026.

The investigation demonstrates that the loss of funds was not due to a sophisticated external hack, but rather the result of **gross negligence**, **concealed centralization risks**, and **ignored indicators of compromise**.

## 📂 Repository Structure

### 📄 Evidence & Documentation
- **`/articles`**: In-depth technical analysis and investigative reports detailing the anatomy of the exploit.
- **`/refs`**: Structured evidentiary references used in legal memorandums. Includes archived documentation, on-chain data snapshots, and verified PoC outputs.
- **`/reports`**: Formal forensic reports tailored for legal counsel, law enforcement, and affected stakeholders.

### 🛠️ Technical Proofs-of-Concept (PoC)
- **`/exploit_test`**: Demonstrates the **Single Point of Failure** in the Drift multisig configuration. 
  - Shows how the `Config Authority` could unilaterally bypass the 2-of-5 consensus.
  - Uses production binaries and patched mainnet state to reproduce the administrative takeover.
- **`/nonce_check_test`**: Demonstrates the **detectability** of the compromise prior to the incident.
  - A simple RPC-based scanner that identifies unauthorized Durable Nonce accounts linked to council members.
  - Proves that the "red flags" were visible on-chain for 39+ hours before the migration.

## ⚖️ Legal & Technical Context

This repository supports the materials, which outlines:
- Breach of Fiduciary Duty
- Material Misrepresentation of Security Architecture
- Gross Negligence in Governance Procedures