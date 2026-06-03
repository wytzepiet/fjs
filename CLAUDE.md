# fjs (wytzepiet fork) — maintainer notes

Soft fork of `fluttercandies/fjs` (QuickJS-via-Rust engine powering solid-fuse).
Goal: stay minimally diverged, upstream everything.

Remotes: `origin` = wytzepiet/fjs, `upstream` = fluttercandies/fjs.

> **solid-fuse vendors this repo's `solid-fuse` branch in-tree via git subtree**
> (at `packages/solid-fuse/dart/lib/src/fjs` there). This repo stays the source
> of truth — edit fjs here, not in the vendored copy. Changes reach the framework
> when someone runs `make sync-fjs` in solid-fuse, not via a pub bump.

## Branches

- **`main`** — mirrors `upstream/main` exactly. Never commit here.
- **`solid-fuse`** — the integration branch the framework consumes. Holds
  fork-only files (these notes, agent docs); never put them on `feat/*`.
- **`feat/<name>`** — one per fix, branched **off `main`**. Additive,
  upstreamable commits only.

## Per-fix flow

1. `git checkout main && git checkout -b feat/<name>`
2. Implement additively, commit (with `Co-Authored-By` trailer), push to origin.
3. *Optional* cross-fork PR to upstream — **drafts only, ask first**:
   `gh pr create --draft --repo fluttercandies/fjs --base main --head wytzepiet:feat/<name>`
4. Integrate into `solid-fuse` by **merging** with a merge commit — never
   cherry-pick (cherry-pick duplicates commits and loses merge tracking):
   `git checkout solid-fuse && git merge --no-ff feat/<name>`
   - If the fix touches fork-only code (e.g. the `start_drive` driver), resolve
     the conflict in the merge and keep any fork-only adaptation on `solid-fuse`,
     not on the feat branch.

## Build / codegen

- Native libs build via **cargokit**; precompiled binaries are keyed by a
  source-hash, so any Rust change cleanly falls back to a local `cargo build`.
- Tests: `cargo test --manifest-path libfjs/Cargo.toml`.
- After adding a bridged `pub async fn` to an opaque type, regenerate with
  `flutter_rust_bridge_codegen` **2.12.0**.
- Dev builds optimize native deps (`[profile.dev.package."*"]`) so QuickJS isn't
  compiled unoptimized under `flutter run` — keeps the dev stack ceiling near
  release.
