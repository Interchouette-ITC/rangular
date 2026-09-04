# Supply-chain checks

Local and CI gates for Rust dependencies.

Requires once: `cargo install cargo-audit cargo-deny`

```bash
make audit   # cargo audit
make deny    # cargo deny check (deny.toml)
```

Add a documented ignore only when an advisory is accepted with a written reason.

## GitHub Dependabot

Public-repo free settings (org): Dependabot alerts, Dependabot security updates, secret scanning, and push protection.

Version bumps: `.github/dependabot.yml` (weekly Cargo and Actions). Review those PRs like any other dependency change; local `make audit` / `make deny` remain the merge gates.
