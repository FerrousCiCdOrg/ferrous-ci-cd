# Ferrous CI/CD branding

Issue: https://github.com/FerrousCiCdOrg/ferrous-ci-cd/issues/90

The project mark combines a rust-orange forge/F silhouette with a pipeline
arrow on a dark navy background. The original was generated with the AGY CLI
using its `image-generator` tool. The PNG and ICO exports retain that design.

| Asset | Dimensions | Purpose |
| --- | --- | --- |
| `ferrous-icon.png` | 1024 × 1024 | Original generated mark |
| `github-avatar.png` | 512 × 512 | GitHub organization profile upload |
| `favicon-16.png` | 16 × 16 | Small browser icon |
| `favicon-32.png` | 32 × 32 | Browser icon |
| `favicon-48.png` | 48 × 48 | Large browser icon |
| `favicon.ico` | 16, 32, 48 | Multi-resolution browser icon |

## Organization profile setup

As an organization owner, visit
https://github.com/organizations/FerrousCiCdOrg/settings/profile and use
**Upload new picture** to upload `github-avatar.png`. Keep the full square
image when cropping so the mark retains its padding.

The public GitHub organization API does not provide an avatar upload field.
The live organization avatar has not been changed through the CLI. The
repository README uses `github-avatar.png`; the favicon files are prepared
for a future web UI and are not served by the current API-only application.

Official instructions:
https://docs.github.com/en/organizations/collaborating-with-groups-in-organizations/customizing-your-organizations-profile#uploading-an-image
