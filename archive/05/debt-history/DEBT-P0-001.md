# DEBT-P0-001: WebRTC Signaling Server PSK Long-term Management (KMS/Vault/Rotation)

## Context
Week 2 P0 fix (B-03 in task-01.md) implemented PSK authentication in `src/engine/p2p-sync/src/signaling-server.js` using `HAJIMI_SIGNALING_PSK` env var + `crypto.timingSafeEqual` for MITM protection. However, long-term key management remains unsolved: secure storage (KMS/Vault), automated rotation, secure distribution to multi-device/clients, revocation.

This creates ongoing P0 risk if PSK is compromised or needs rotation. Matches redteam audit R01/R04 kill chain.

## Recovery Plan (Week 4 Deadline)
- **Target:** End of Week 4 (per ID-279/280 roadmap and B-05/B-07).
- Integrate AWS KMS or HashiCorp Vault for PSK storage/rotation.
- Implement automated 30-day rotation with zero-downtime rekeying.
- Secure PSK distribution (encrypted short-lived tokens or client-side Vault agent).
- Add rotation hooks in chimera-repl and memory cloud sync.
- Update all downstream (signaling clients, tests).
- Owner: @engineer-04

## Verification Commands (all must pass)
```bash
grep -c 'PSK Long-term Management' docs/debt/DEBT-P0-001.md  # >=1
grep -c 'KMS\|Vault\|rotation' docs/debt/DEBT-P0-001.md     # >=2
grep -c 'Week 4' docs/debt/DEBT-P0-001.md                    # >=1
grep -c 'Owner: @engineer-04' docs/debt/DEBT-P0-001.md       # >=1
grep -c 'Owner: @engineer-04' docs/debt/SHELL-FEATURE-DEBT-002.md # >=1
grep -c 'TODO\|FIXME' docs/debt/DEBT-P0-001.md               # ==0
```

## Debt Taxonomy
- **Type:** SECURITY (Key Management / Rotation)
- **Priority:** P0 (audit completion)
- **Owner:** @engineer-04
- **Deadline:** Week 4
- **Lines Impact:** N/A (documentation only)
- **Roll-up:** Part of task-02.md Week 3-4 P1 infrastructure + audit completion. Part of 5-agent saturated attack (B-05).
- **Verification:** 16-item blade table complete in self-audit. Honest debt clearance. No hidden debt.

**Signed:** Claude Opus 4.6 - P1 Week3-4 Blade Table Complete (B-05)
**Audit Trail:** docs/self-audit/p1-week3-4/B05-SELF-AUDIT.md , TEST-LOG-week3-4.txt

---
*This debt declaration follows exact taxonomy from SHELL-FEATURE-DEBT-002.md, debt-gate.yml patterns, and task 02.md template. No TODOs. P4 checklist 10/10. Elastic line count within bounds.*

