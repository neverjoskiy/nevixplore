# NevIxplore

A fast, beautiful, ad-free file manager for Windows. Built with Rust and Tauri 2.

> **Status: pre-alpha.** PR-0 lands the scaffold, CI, and a window that opens. The file
> list, search, and preview arrive in later PRs. There is nothing useful to install yet.

<!-- Screenshot placeholder: replace with docs/screenshot.png once the list lands in PR-1. -->
<p align="center"><em>Screenshot coming with the first real file list.</em></p>

## What it will do

NevIxplore is a spiritual successor to File Explorer that fixes its four worst problems.

- **Instant on huge directories.** Virtualized list, no full scans before the first paint.
- **Search that actually works.** Live name matching as you type, plus optional content search.
- **Preview without opening anything.** Images, text, code, and media in a side pane.
- **No ads, no nagging, no telemetry.** Your files are your business.

## Requirements

- Windows 10 21H2 or newer, or Windows 11 (x86_64).
- WebView2, which ships with Windows 11 and current Windows 10.

## Install

No release exists yet. Once `v0.0.1` is tagged, a portable `.exe` and an installer will be
attached to the GitHub Release. Builds are produced only by CI; see
[docs/BUILDING.md](docs/BUILDING.md).

## Development

Nothing is installed on a contributor machine by design: **all compilation, linting, testing
and packaging run in GitHub Actions.** Read [docs/BUILDING.md](docs/BUILDING.md) before you
start.

## Roadmap

Work proceeds in reviewable PRs. Each PR is green in CI before the next begins.

| Stage | Scope |
|---|---|
| PR-0 | Scaffold, CI, a window that opens |
| PR-1 | Directory listing, virtualized file list, navigation |
| PR-2 | Search, filtering, and sorting |
| PR-3 | Preview pane |
| PR-4 | File operations, with the Recycle Bin as the default |
| Later | Tabs, indexing, plugins, benchmarking |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Security reports go through
[SECURITY.md](SECURITY.md), never a public issue.

## License

MIT. See [LICENSE](LICENSE).
