## What and why

<!-- What this changes, and why. Link the issue if there is one. -->

## Checklist

- [ ] `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] The end-to-end tests in `tests/e2e` pass, with a new check where it fits
- [ ] Shared behaviour is changed in the base (`Phygital-Key/phygital-key`), never here
- [ ] No new dependency, or it was discussed in an issue first
- [ ] Docs updated where people will notice the change, and `CHANGELOG.md` has a line
- [ ] No em dashes in docs or messages
- [ ] No secrets, tokens, real private keys or release keys anywhere in the change
- [ ] The NOTICE credit is unchanged

## Effect on safety

<!-- Does this weaken a defence listed in the base's docs/threat-model.md? Which test proves it still holds? -->
