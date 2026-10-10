# Security policy

## Supported version

Only the newest released version receives security fixes.

| Version | Supported |
|---------|-----------|
| v0.0.6 (latest) | yes |
| older | no, please update |

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's private vulnerability reporting:
<https://github.com/michalsarna/LightCreator/security/advisories/new>

Include the version, your operating system, what you did and what happened. Files that trigger the problem
(an SVG, PDF / Illustrator, image or project file) are very helpful. You can expect an answer within a few
days; a fix is released as a new version.

## What counts

LightCreator opens files from other people (SVG, PDF / Illustrator, bitmaps, `.lcr` projects) and talks to
hardware over a serial port. Reports about crashes, hangs or memory use when opening a crafted file, and about
anything that could send unintended commands to a laser controller, are in scope.

## Safety reminder

Lasers are dangerous. Test new jobs at low power, wear eye protection and never leave a running machine
unattended. The material library only contains starting values.
