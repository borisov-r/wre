# Release Process

This document explains how releases are created for the WRE (Wireless Rotary Encoder) project.

## Automatic Release System

The project uses **automatic date-based releases** that are created whenever code is successfully built on the `main` or `master` branch.

### How It Works

1. **Trigger**: When code is pushed to the `main` or `master` branch
2. **Build**: The CI workflow automatically builds the ESP32 firmware in release mode
3. **Version**: A date-based version tag is automatically generated (e.g., `v2026.01.29`)
4. **Release**: A GitHub Release is automatically created with:
   - Pre-built firmware binary
   - README, QUICKSTART, and NODEMCU_SETUP documentation
   - Installation instructions
   - Automatic release notes from commits

### Version Naming Convention

Releases use **date-based versioning** in the format:

- `v2026.01.29` - First release of the day
- `v2026.01.29.1` - Second release of the day (if multiple builds occur)
- `v2026.01.29.2` - Third release of the day, etc.

**Note:** Dates are in **UTC timezone**. If you commit late in your local day, the release might have the next day's date in UTC.

This ensures:
- ✅ Every successful build on main/master gets a release
- ✅ Versions are chronological and easy to understand
- ✅ Multiple releases per day are supported
- ✅ No manual intervention required
- ✅ Consistent timezone (UTC) for all releases

## Finding Releases

All releases are available on the [Releases page](../../releases).

The latest release is always the most recent date.

## What's in a Release

Each automatic release includes:

1. **Firmware Binary**: `wre-esp32-v2026.01.29` (ready to flash)
2. **Documentation**: README, QUICKSTART, and NODEMCU_SETUP guides
3. **Installation Instructions**: How to flash the firmware
4. **Release Notes**: Automatically generated from commit messages
5. **Commit Reference**: Links to the exact code that was built

## Installation

Download the latest firmware from the [Releases page](../../releases) and flash it:

```bash
# Download the latest release firmware
# Then flash it to your ESP32
espflash flash wre-esp32-v2026.01.29
```

See [QUICKSTART.md](QUICKSTART.md) for complete setup instructions.

## For Developers

### No Manual Steps Required

Unlike traditional release systems, you don't need to:
- ❌ Create version tags manually
- ❌ Run release scripts
- ❌ Decide on version numbers
- ❌ Manually trigger releases

### Every Main Branch Build Creates a Release

Simply merge your changes to the `main` or `master` branch, and a release is automatically created when the build succeeds.

### Workflow Details

The CI workflow (`.github/workflows/ci.yml`) handles everything:

1. **Build Check**: Runs on all pushes and PRs
2. **Release Creation**: Only runs for pushes to main/master (not PRs)
3. **Versioning**: Automatically generates date-based tags
4. **Publishing**: Creates GitHub Release with all artifacts

### Continuous Deployment

This automatic release system enables continuous deployment:
- Every commit to main/master is automatically released
- Users always have access to the latest stable build
- No release bottlenecks or manual processes
- Full traceability from release to source code

## Release Frequency

Releases are created automatically for every successful build on main/master:

- **Development pace**: If you push 5 commits to main in one day, you'll get `v2026.01.29`, `v2026.01.29.1`, `v2026.01.29.2`, `v2026.01.29.3`, `v2026.01.29.4`
- **Production readiness**: Only merge to main/master when code is ready for release
- **Quality control**: Use PRs and branch protection to ensure main/master only contains production-ready code

## Troubleshooting

### Release wasn't created after push to main

1. Check the [Actions tab](../../actions) - did the CI workflow succeed?
2. Verify you pushed to `main` or `master` branch (not a feature branch)
3. Check workflow logs for any errors in the release steps

### How do I create a specific version number?

The system uses automatic date-based versioning. You cannot create custom version numbers. The date-based approach ensures:
- Chronological ordering
- No version conflicts
- Automated workflow
- Clear release timeline

### Can I manually create releases?

The automatic system replaces manual releases. All releases should come from the CI workflow to ensure:
- Consistent build environment
- Verified builds (tests pass)
- Complete documentation
- Traceable to source commits

## Migration from Manual Releases

Previous versions of this project used manual tag-based releases. This has been replaced with automatic releases for:

- ✅ **Faster releases**: No manual steps
- ✅ **Consistency**: Every build is released the same way
- ✅ **Traceability**: Clear connection between releases and commits
- ✅ **Simplicity**: No need to decide version numbers or run scripts

If you have the old `create-release.sh` script, it's no longer needed and can be removed.

## Additional Resources

- [GitHub Releases Documentation](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [CI/CD Best Practices](https://docs.github.com/en/actions/guides)
- [Continuous Deployment](https://en.wikipedia.org/wiki/Continuous_deployment)
