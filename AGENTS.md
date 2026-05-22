# AGENTS.md — Murmur AI Agent Guide

> This file defines how AI agents work on the Murmur project.
> Maintained by **Satoshi** (🥷 OpenClaw).

## What This Is

Murmur is a privacy-first Reddit frontend proxy — browse Reddit without tracking, ads, or data collection. Written in Rust.

**Stack:** Rust 2021 edition, Axum 0.7, Askama 0.12 (server-side templates), reqwest 0.12, moka cache, Reddit OAuth2 (app-only for browsing, user OAuth for voting/commenting).

## Agent Principles

1. **Read first** — before coding, read the relevant source files, templates, and CSS. Understand the handlers, models, and the Reddit API flow.
2. **Build & typecheck after every change** — `cargo build` or `cargo check`. No broken builds.
3. **No guessing in a browser context** — layout/frontend bugs need a headless Chrome screenshot to verify, not server-side analysis.
4. **Keep it clean** — no commented-out code, no `println!` in production, no dead routes.
5. **Document as you go** — update `changes.md` with every meaningful change.
6. **One change at a time** — avoid batch edits that touch multiple systems. Test each change before the next.

## File Reference

| File | Purpose |
|------|---------|
| `AGENTS.md` | (this file) Agent collaboration guide |
| `changes.md` | Change log — what changed, when, by whom |
| `README.md` | Project overview, build instructions, config |
| `review.md` | UI/UX & design review (27 findings catalogued) |

## Architecture Notes

- **No database** — Murmur is stateless. Reddit API responses are cached in moka (in-memory) or optionally Redis. User sessions (for OAuth) use encrypted cookies.
- **Templates are Askama** — compiled at build time. Template structs in `src/templates.rs`. Changing a template means recompiling.
- **Reddit API access** — app-only OAuth for browsing (no user token needed), user OAuth for voting/commenting/saving.
- **Media proxy** — images/videos can be proxied through the server to avoid Reddit's tracking domains.
- **CSS design system** — complete custom design system in `static/css/design-system.css`. Uses CSS custom properties for theming. No Tailwind, no framework.

## Git

- Remote uses SSH: `git@github.com-murmur:skulls206-creator/murmur.git`
- Deploy key at `~/.ssh/murmur_key`
- Push to `main` when changes are complete and tested.

## Red Lines

- Don't push broken builds. Test before committing: `cargo build` must pass.
- Don't expose Reddit OAuth tokens or client secrets in commits.
- Don't remove privacy filtering (header stripping, user-agent spoofing) from the Reddit API proxy.
- If upgrading dependencies, verify no breaking changes to request/response handling.
