# CHANGES — Murmur

> Shared change log for AI agents. Newest entry on top.

---

## 2026-05-22 — Repository init & SSH deploy key setup
**Author:** Satoshi
**Scope:** repo infrastructure
**Changes:**
- Generated `~/.ssh/murmur_key` (Ed25519 deploy key)
- Added write-enabled deploy key to `skulls206-creator/murmur` GitHub repo (key ID: 152283643)
- Added SSH config host `github.com-murmur` → `~/.ssh/murmur_key`
- Switched git remote from HTTPS+PAT to SSH: `git@github.com-murmur:skulls206-creator/murmur.git`
- Created `AGENTS.md` and `changes.md` following the standard project pattern

## 2026-05-22 — UI/UX & Design Review completed
**Author:** Satoshi
**Scope:** full project audit
**Changes:**
- Thorough review of all 27 findings across:
  - 3 BREAKING (broken `/api/save`, `--space-14` missing, sidebar toggle `hidden` attr)
  - 7 HIGH (comment depth colors, duplicated inline scripts, missing skeletons, infinite scroll edge cases, etc.)
  - 7 MEDIUM, 6 LOW, 4 SUGGESTIONS
- Full review saved to `review.md`
