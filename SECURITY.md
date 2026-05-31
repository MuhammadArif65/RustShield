# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**Why not open a public GitHub Issue?**
Although this is an Open Source project, FerrumWard is fundamentally a security software. If someone discovers a security flaw (Zero-Day Vulnerability) and reports it publicly in the Issues tab, malicious actors and game pirates can immediately exploit it to compromise games before we have a chance to release a patch.

Therefore, we follow the global security industry standard known as **Coordinated Vulnerability Disclosure (CVD)**. We ask you to report security vulnerabilities privately so we can patch them *before* they are made public.

If you discover a security vulnerability in FerrumWard, **please do NOT open a public GitHub issue**.
Instead, please report it responsibly via private email:

- **Email:** muhamadarif7566@gmail.com
- **Subject:** `[SECURITY] Brief description of vulnerability`

### What to include:
- A clear description of the vulnerability
- Steps to reproduce the issue
- The potential impact
- Any suggested fix (optional but appreciated)

### Response timeline:
- **Acknowledgement:** Within 48 hours
- **Initial assessment:** Within 7 days
- **Fix or mitigation:** Within 30 days for critical issues

We will credit you in the release notes (unless you prefer to remain anonymous).

## Scope

The following are in scope for security reports:
- Bypasses of anti-debug, anti-VM, or anti-tamper protections
- Weaknesses in the Ed25519 license validation flow
- Memory safety issues in `unsafe` blocks
- Cryptographic weaknesses in HWID generation or AES-GCM usage
- Information leaks through error messages

## Out of Scope

- Attacks requiring physical access to the target machine
- Social engineering
- Denial of service against the protected application itself


<!-- -->
