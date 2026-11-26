# Changelog

All notable changes to **iPort** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-25

### Added
- Cross-platform TCP & UDP listening port scanner for Linux, macOS, and Windows with 2-second live polling snapshots.
- OS vs. 3rd-party application classifier that hides core OS daemons by default while surfacing dev servers, Docker containers, tunnels, background services, and third-party apps.
- Smart framework & runtime detection (Next.js, Vite, React, Node/Bun/Deno, Python/FastAPI/Uvicorn/Django/Flask, Go, Rust, PostgreSQL, Redis, MongoDB, and more) plus Docker container correlation via `bollard`.
- High-density searchable, sortable desktop table with quick filter chips (`All`, `Dev`, `Docker`, `Services`, `Tunnels`, `3rd-Party`, `System`), keyboard navigation, and port detail inspector.
- Developer quick actions: open in browser, copy URL/port, reveal working directory, graceful stop (`SIGTERM`), and two-step force kill (`SIGKILL`).
- System-adaptive Light/Dark themes with GNOME/Yaru accent color synchronization.
- Cross-platform packaging and automated GitHub Releases for Linux (`.deb`, `.rpm`, `.AppImage`), macOS (`.dmg` Apple Silicon & Intel), and Windows (`.msi`, `-setup.exe`).

[Unreleased]: https://github.com/yegekucuk/iPort/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yegekucuk/iPort/releases/tag/v0.1.0
