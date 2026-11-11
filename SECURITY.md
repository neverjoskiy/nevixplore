# Security Policy

## What NevIxplore does with your data

**Nothing leaves your machine.** NevIxplore has no telemetry, no analytics, no crash
reporting, and no update pings that identify you. There is no account, no sign-in, and no
server component. The application makes no network requests of its own; the only network
traffic associated with the project is a GitHub Release download you initiate yourself.

Settings live in `%APPDATA%\NevIxplore\settings.toml`. Nothing else is persisted between
launches, and no file contents are ever copied out of the directory you are browsing.

## Supported versions

NevIxplore is pre-alpha and has no supported release yet. Security fixes target `main` and,
once they exist, the latest tagged release.

| Version | Supported |
|---|---|
| `main` (unreleased) | Yes |
| Tagged releases | None yet |

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Use GitHub's private vulnerability reporting on this repository
(*Security* tab, then *Report a vulnerability*). If that is unavailable, contact the
maintainer directly through their GitHub profile.

Please include:

- The version or commit you tested.
- Steps to reproduce, and whether the issue needs a crafted directory, filename, or archive.
- The impact you believe it has.

You can expect an acknowledgement within 72 hours and an assessment within 7 days. Please
give a fix a reasonable window before public disclosure; we will credit you in the release
notes unless you prefer otherwise.

## Scope

Especially interesting for this project:

- Path handling that escapes the browsed directory, including `..`, UNC paths, junctions, and
  paths longer than `MAX_PATH`.
- Anything that turns a filename, archive entry, or file metadata into code execution.
- `unsafe` blocks in the Win32 FFI layer — an incorrect lifetime or buffer length there is a
  memory-safety bug.
- Writes that destroy data the user did not ask to delete. The Recycle Bin is the default
  precisely so a bug is recoverable.

Out of scope: issues that require an already-compromised machine, and the behaviour of
Windows itself.
