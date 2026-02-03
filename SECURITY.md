# Security

## Supported versions

We release patches for security issues. Currently supported versions are listed in the [releases](https://github.com/dimension/dimension/releases) page.

## Reporting a vulnerability

If you believe you have found a security vulnerability, please report it responsibly:

1. **Do not** open a public issue.
2. Email the maintainers (see repository contacts or `Cargo.toml` authors) with:
   - A description of the vulnerability and how it might be exploited.
   - Steps to reproduce (if possible).
   - Your name/handle for acknowledgment (optional).
3. We will respond as soon as we can and will work with you to understand and address the issue.
4. We may issue a fix and coordinate disclosure after affected users have had time to update.

Thank you for helping keep Dimension and its users safe.

## Security update process

- Dependencies are monitored via [Dependabot](.github/dependabot.yml) and CI runs `cargo audit` for known advisories.
- We address reported vulnerabilities and dependency advisories in a timely manner and will credit reporters when they wish.
