# Conventions

Code, file names, folders and commits are written in English. Text shown to
the user is Brazilian Portuguese and lives in one place per binary
(`i18n/pt_br.rs` in the interface). The names and descriptions of cleanup
rules are part of the rule catalog (`resources/cleanup_rules.toml`).

## Documentation

- The README and the guides in `docs/guide/` exist in English and Portuguese.
  The English file is `name.md`; its translation sits next to it as
  `name.pt-BR.md`. Change both in the same pull request.
- Documents for contributors (this one, the architecture, the security policy,
  the decision records) are in English only.
- Documentation states only what is true today. A planned feature is labeled
  as planned, an idea as an idea, and anything not verified on the real thing
  is listed as not verified.
- When a guide names a button or a label, it quotes the text of the interface
  exactly as it is in `i18n/pt_br.rs`. The English guides add the meaning in
  parentheses the first time.
- Images live in `docs/assets/`. They are produced by
  `docs/assets/capture/make_assets.py` from the real app in its development
  capture mode, using a demonstration home folder, so no personal path or
  program appears in them. Captions say they are screenshots of the app with
  demonstration data. That script is a documentation tool written in Python;
  it is not part of the product and is not built by Cargo.
- Examples use `~/…` paths and placeholder names. No real user name, home
  folder name or e-mail address goes into the documentation.

## Naming

| Item | Rule | Example |
|---|---|---|
| Crate | kebab-case, `diskmarshal-` prefix | `diskmarshal-domain` |
| Folder and `.rs` file | snake_case; one main type per file, same name | `duplicate_group.rs` holds `DuplicateGroup` |
| Module with children | `name.rs` plus folder `name/`; never `mod.rs` | `scan.rs`, `scan/file_tree.rs` |
| Struct, enum, trait | UpperCamelCase | `FileTree`, `RiskLevel` |
| Port (trait) | noun for a capability | `ContentHasher` |
| Adapter | technology + port name | `Blake3ContentHasher` |
| Use case | verb + object, single `execute` method | `ScanDisk` in `scan_disk.rs` |
| Function, variable | snake_case; booleans start with `is_`, `has_`, `can_` | `is_cloud_only` |
| Constant | SCREAMING_SNAKE_CASE | `STAT_BLOCK_BYTES` |
| Error | one `...Error` enum per crate, built with `thiserror` | `ApplicationError` |
| Cleanup rule id | lowercase words joined by hyphens; the privileged helper refuses anything else | `system-caches` |
| Test | a sentence stating the behavior | `never_follows_symbolic_links` |
| Document | kebab-case; decision records are numbered | `docs/adr/0002-one-shot-privilege-elevation.md` |
| Commit | [Conventional Commits](https://www.conventionalcommits.org/) | `feat(scan): add bulk walker` |

## No duplicated code

- **Workspace-level configuration.** Version, edition, license, dependency
  versions and lints are declared once in the root `Cargo.toml` and inherited
  with `key.workspace = true`.
- **Rules as data.** A new cache location or ecosystem is a new TOML entry in
  `crates/diskmarshal-infrastructure/resources/`. [CONTRIBUTING.md](../CONTRIBUTING.md#adding-a-cleanup-rule-or-an-ecosystem)
  shows how.
- **Shared interface components.** Lists with selection, risk badges, action
  footers, progress panels and the removal flow (confirm, run, report) are
  single components reused by every screen. Duplicates and photos share the
  same group review.
- **Single sources.** Colors, glow and durations in `theme/tokens.rs`; texts in
  `i18n/pt_br.rs`; size formatting in `ByteSize`; fixtures in
  `diskmarshal-test-support`.
- **Rule of thumb.** The second time the same logic appears, extract it.

## Code quality

- Public items have documentation comments (`missing_docs` is enforced).
- No `unwrap`, `expect` or `panic!` outside tests. Errors are typed with
  `thiserror`; `anyhow` is used only in binaries.
- Sizes and counters use saturating arithmetic.
- Unit tests sit next to the code in a `tests` module; tests that need real
  files use `TempTree` from `diskmarshal-test-support`.

## Quality gate

```bash
cargo xtask check
```

It runs `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` and
`cargo deny check`. CI runs the same command. A change is not done while it fails.

The other project tasks build the app: `cargo xtask bundle` assembles and
signs `dist/DiskMarshal.app`, and `cargo xtask install` also copies it to
`/Applications` (or to the folder given as an argument).
