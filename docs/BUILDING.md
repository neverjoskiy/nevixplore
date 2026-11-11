# Building NevIxplore

**All builds happen in GitHub Actions.** No Rust toolchain, Node install, or WebView2
bootstrapper is installed on a contributor machine. This document explains how to get a build
out of CI, and how the three workflows differ.

## Why CI-only

The project targets Windows 10/11 with the MSVC toolchain and WebView2. Keeping the build in
CI means every binary is produced in the same environment, reproducibility is free, and a
contributor needs nothing but `git`. The cost is that you cannot run the app the moment you
clone; the benefit is that "works on my machine" is not a category of bug here.

## The three workflows

| Workflow      | Trigger                    | Produces                                    |
| ------------- | -------------------------- | ------------------------------------------- |
| `ci.yml`      | push to `main`, every PR   | checks, plus a portable `.exe` artifact     |
| `release.yml` | push a `v*` tag            | portable zip and an NSIS installer          |
| `nightly.yml` | 03:00 UTC daily, or manual | the `nightly` prerelease, 7 builds retained |

### `ci.yml`

Three jobs run on `windows-latest`:

- **`rust`** — `cargo fmt --all --check`, then
  `cargo clippy --workspace --all-targets -- -D warnings`, then `cargo test --workspace --locked`.
- **`frontend`** — `npm ci`, `npm run lint`, `npx prettier --check .`, `npm run build`.
- **`bundle`** — waits for both, then `npm run tauri build -- --no-bundle` and uploads
  `nevixplore.exe` as the `nevixplore-portable` artifact.

Download the artifact from the workflow run page to try a build without making a release.

### `release.yml`

Pushing a tag is the whole interface:

```sh
git tag v0.0.1
git push origin v0.0.1
```

CI runs a full `tauri build`, which produces the app and an NSIS installer from the
`bundle` section of `src-tauri/tauri.conf.json`. It publishes two assets to a GitHub Release
with generated notes: `NevIxplore-<tag>-portable.zip` (the bare `.exe`) and
`NevIxplore-<tag>-setup.exe` (the installer). The step fails loudly if the installer is
missing, so a release cannot silently ship the zip alone.

The spec calls for Inno Setup in this workflow. PR-0 uses the NSIS target instead because
Tauri already configures it and it needs no `.iss` script to exist; packaging is revisited in
PR-14, where the installer gets real attention.

### `nightly.yml`

Runs daily from `main` and refreshes a rolling prerelease tagged `nightly`. Assets are named
`NevIxplore-nightly-<yyyymmdd>-portable.zip`; the seven newest are kept and older ones are
deleted. Start it by hand from the Actions tab when you need a build sooner.

## Local checks that do not need a compiler

A formatter and a linter are not builds, and both are worth running:

```sh
cargo fmt --all              # format
npm run lint                 # ESLint
npx prettier --check .       # formatting
```

If you happen to have a full toolchain, `cargo clippy --workspace --all-targets` and
`npm run build` work locally too. CI still decides.

## Releasing

1. Confirm `main` is green.
2. Tag and push: `git tag v0.1.0 && git push origin v0.1.0`.
3. CI publishes the release. Edit the generated notes if the summary needs context.

## Troubleshooting

- **`link.exe not found`** — you are trying to build locally without the MSVC build tools.
  Push the branch instead; CI has them.
- **`frontendDist` missing during `tauri build`** — `npm run build` must run first, which the
  workflows do. `beforeBuildCommand` handles it when you run the Tauri CLI directly.
- **Bundle job cannot find the exe** — the binary is named after the crate (`nevixplore.exe`)
  and lives in the workspace root `target/release/`, not `src-tauri/target/release/`, because
  the root `Cargo.toml` makes the workspace own the target directory.
