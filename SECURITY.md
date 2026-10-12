# Security Policy

## Supported versions

Only the **latest release** receives security fixes. Oasis ships as a single
binary, through Colony and the release page, so the fix for a vulnerability is
the next release, not a patch to an older one.

## Reporting a vulnerability

Please report vulnerabilities **privately** via
[GitHub Security Advisories](https://github.com/Project-Colony/Oasis/security/advisories/new)
("Report a vulnerability"). Do not open a public issue for exploitable bugs.

Include the version (`oasis --version`), the platform, and the steps, response
or file that reproduce it. The report stays private until a fixed release is
out.

## Scope

Oasis takes input from three places, and these are the reports of most interest:

- **Network responses.** Each lookup parses JSON from ipapi.co, ipwho.is and
  Open-Meteo over HTTPS. A response that crashes Oasis, hangs it, or makes it
  do anything beyond showing a notification or an error is in scope, as is
  anything that weakens TLS certificate checks.
- **Notification rendering.** The place names in a notification come from
  those responses. On Linux the body is read as markup and Oasis escapes it;
  text that gets through as markup, or that changes what the notification
  does on any OS, is in scope.
- **Config parsing.** A `config.toml`, or the one-time copy of a config from an
  earlier version, that makes Oasis do more than fall back to its defaults with
  a warning.

Out of scope: the accuracy of IP geolocation, the availability of the
third-party services, and vulnerabilities in those services themselves (report
those to them).

## Verifying releases

Every release asset is signed with the Project Colony ed25519 release key. The
[Code signing policy](README.md#code-signing-policy) shows how to check a
download.
