# Security policy

## Supported version

Only the newest released version receives security fixes.

| Version | Supported |
|---------|-----------|
| v0.3.8 (latest) | yes |
| v0.3.7 | no, please update |
| v0.3.6 | no, please update |
| v0.3.5 | no, please update |
| v0.3.4 | no, please update |
| v0.3.3 | no, please update |
| v0.3.2 | no, please update |
| v0.3.1 | no, please update |
| v0.3.0 | no, please update |
| v0.2.9 | no, please update |
| v0.2.1 | no, please update |
| older | no, please update |

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's private vulnerability reporting:
<https://github.com/michalsarna/LightCreator/security/advisories/new>

Include the version, your operating system, what you did and what happened. Files that trigger the problem
(an SVG, PDF / Illustrator, image or project file) are very helpful. You can expect an answer within a few
days; a fix is released as a new version.

## What counts

LightCreator opens files from other people (SVG, PDF / Illustrator, bitmaps, `.cre` projects) and talks to
hardware over a serial port. Reports about crashes, hangs or memory use when opening a crafted file, and about
anything that could send unintended commands to a laser controller, are in scope.

## Safety reminder

Lasers are dangerous. Test new jobs at low power, wear eye protection and never leave a running machine
unattended. The material library only contains starting values.
