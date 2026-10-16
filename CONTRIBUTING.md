# Contributing to NanoPrint

Thanks for helping. Everyone taking part follows the [code of conduct](CODE_OF_CONDUCT.md). The
project rules live in [CLAUDE.md](CLAUDE.md) and apply to every contributor, human or AI. In short:

- **Clean room.** Never copy code from QZ Tray or from projects whose license has not been
  approved. Protocol compatibility comes from public documentation and black-box testing.
- **Security first.** Read the "Secure by default" maxims and the
  [threat model](docs/threat-model.md) before touching the print server, the permission flow or the
  updater.
- **Scope.** NanoPrint transports print jobs. Invoicing, tax rules and receipt templates belong to
  the applications that use it.
- **English** for code, comments, docs and commits. UI text goes in `locales/` (English and
  Spanish).

> QZ Tray and qz-tray.js are products of QZ Industries, LLC; QZ Tray is its trademark. NanoPrint is an independent project, not affiliated with or endorsed by QZ Industries, and mentions them only to describe compatibility.

## Before opening a pull request

Run what CI runs:

```sh
npm run format:check && npm run lint && npm run check && npm test
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace   # needs `npm run build` first
```

Update `docs/`, the README and `CHANGELOG.md` when the change is visible to users or to web pages.

## Dependencies

- Add a dependency only when it pays for itself, with a license that `deny.toml` (Rust) or
  `scripts/third-party-notices.mjs` (JavaScript) accepts.
- npm: packages that end up inside the app go in `dependencies`; build and test tools in
  `devDependencies`. The license notices are built from that split.
- Install scripts are disabled (`.npmrc`). A package that cannot work without one needs a
  discussion first.
- `npm run notices` (with [cargo-about](https://github.com/EmbarkStudios/cargo-about) installed)
  checks every license; CI runs it too.

## Developer Certificate of Origin

Every commit must be signed off:

```sh
git commit -s -m "feat(tray): show the listening address"
```

`-s` adds a `Signed-off-by: Your Name <you@example.com>` line. With it you certify the
[Developer Certificate of Origin](https://developercertificate.org/): that you wrote the change, or
otherwise have the right to submit it under NanoPrint's license (GPL-3.0-only). It is a statement,
not a contract to sign, and it is how the project keeps track of where every line comes from,
which matters for a clean-room implementation. CI checks every commit of a pull request; to fix a
missing sign-off, run `git commit --amend -s` (or `git rebase --signoff main`) and push again.

Use [Conventional Commits](https://www.conventionalcommits.org/) for the message.

## License of contributions

NanoPrint is licensed under GPL-3.0-only. Contributions are accepted under the same license.
