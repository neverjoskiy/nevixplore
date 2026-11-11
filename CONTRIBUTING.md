# Contributing to NevIxplore

Thanks for helping. This project has an unusual rule that shapes everything below: **no build
tools are installed on contributor machines.** Rust, Node, and packaging all run in GitHub
Actions. That is not a limitation to work around; it is the workflow.

## The loop

1. Open an issue describing the change, or claim an existing one.
2. Branch from `main`: `feat/short-name`, `fix/short-name`, `chore/short-name`.
3. Write the change. Do not run `cargo build`, `cargo test`, `rustup`, or `npm install`
   locally.
4. Push the branch and open a PR. CI runs `fmt`, `clippy -D warnings`, `test`, `lint`,
   `prettier`, `npm run build`, and a release bundle.
5. Fix what CI reports. Review starts once the `rust`, `frontend`, and `bundle` checks pass.
6. Squash-merge. History on `main` is linear.

If you *can* build locally, feel free — just make sure the branch is green in CI, because
that is the only result the project trusts.

## Definition of done

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
  `npm run lint`, `prettier --check`, and `npm run build` all pass in CI.
- No `unwrap()`, `expect()`, or `panic!()` in non-test code outside a documented panic
  boundary. Use `let Some(x) = ... else { return Err(...) };`.
- `unsafe` appears only in the Win32 FFI layer, and only with a `// SAFETY:` comment.
- New behaviour has a test. Tests must pass in a clean container with no network.
- Public items have `///` doc comments. Inline explanatory `//` comments are not used;
  reasoning belongs in `docs/adr/`.
- No `TODO` without an issue number.

## Commit style

Conventional Commits, always. `feat:`, `fix:`, `chore:`, `refactor:`, `docs:`, `test:`,
`perf:`, `ci:`. A commit without a valid prefix fails review.

```
feat(list): virtualize rows and keep scroll position on refresh
```

## Code standards

- **Rust:** edition 2024, `rustfmt` with `max_width = 100` and `imports_granularity = "Crate"`.
  Errors via `thiserror` in libraries and one `AppError` at the Tauri boundary. Blocking disk
  work goes through `tokio::task::spawn_blocking`. No `#[allow(...)]` without a reason on the
  same item.
- **TypeScript:** `strict`, plus `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes`.
  No `any`. Anything crossing the IPC boundary is typed, and the Rust type is the source of
  truth. Function components and hooks only.
- **Layering:** `UI -> commands -> services -> OS`. Commands are thin adapters with no I/O of
  their own.

## Reviewing

Ask for one thing: does this make the next PR easier? Changes that add abstraction before a
second caller exists will be asked to wait.

## Code of conduct

Participation is covered by [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
