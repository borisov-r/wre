# Release Process

This document explains how to create releases for the WRE (Wireless Rotary Encoder) project.

## Understanding the Release System

The project uses GitHub Actions to automatically build and publish releases when you push a version tag. The release workflow:

1. **Triggers** when you push a tag matching the pattern `v*.*.*` (e.g., `v1.0.0`, `v0.2.1`)
2. **Builds** the ESP32 firmware in release mode
3. **Creates** a GitHub Release with:
   - Pre-built firmware binary
   - README, QUICKSTART, and NODEMCU_SETUP documentation
   - Installation instructions

## Why No Releases Are Showing

**The release workflow has never been triggered because no version tags exist in the repository.**

The CI/Build workflow runs successfully on every push, but releases only happen when you create and push a version tag.

## How to Create a Release

### Option 1: Using the Helper Script (Recommended)

The easiest way to create a release is using the provided helper script:

```bash
# Run the interactive release script
./create-release.sh
```

The script will:
1. Check your repository status
2. Show existing tags
3. Prompt for version number (validates semantic versioning)
4. Ask for a release message
5. Create and push the tag automatically
6. Provide links to monitor the release workflow

### Option 2: Manual Tag Creation

If you prefer manual control:

### Step 1: Decide on a Version Number

Follow [Semantic Versioning](https://semver.org/):
- **MAJOR** version (v**X**.0.0) - Incompatible API changes
- **MINOR** version (v0.**X**.0) - New functionality (backwards compatible)
- **PATCH** version (v0.0.**X**) - Bug fixes (backwards compatible)

For the first release, use `v1.0.0` or `v0.1.0` depending on maturity.

### Step 2: Create and Push the Tag

```bash
# Ensure you're on the main branch with latest changes
git checkout main
git pull origin main

# Create an annotated tag with a message
git tag -a v1.0.0 -m "Release version 1.0.0"

# Push the tag to GitHub (this triggers the release workflow)
git push origin v1.0.0
```

### Step 3: Monitor the Release Workflow

1. Go to the [Actions tab](../../actions) in GitHub
2. You should see a "Release" workflow running
3. Wait for it to complete (takes ~10-15 minutes to build ESP32 firmware)

### Step 4: Verify the Release

1. Go to the [Releases page](../../releases)
2. You should see your new release with:
   - Firmware binary named `wre-esp32-v1.0.0`
   - Documentation files
   - Installation instructions

## Quick Reference Commands

```bash
# List existing tags
git tag -l

# Create and push a new release tag
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0

# Delete a tag if you made a mistake (be careful!)
git tag -d v1.0.0                    # Delete locally
git push origin --delete v1.0.0     # Delete from GitHub
```

## Release Checklist

Before creating a release:

- [ ] All tests pass (`cargo test` if applicable)
- [ ] CI workflow is passing on main branch
- [ ] Update CHANGELOG.md (if you have one)
- [ ] Version number follows semantic versioning
- [ ] Tag message describes the release
- [ ] Documentation is up to date

## Automated Release Workflow Details

The `.github/workflows/release.yml` workflow:

1. **Triggers on**: Tags matching `v*.*.*` pattern
2. **Permissions**: Requires `contents: write` to create releases
3. **Build steps**:
   - Sets up Rust with ESP32 toolchain
   - Builds firmware in release mode
   - Verifies build output
4. **Release creation**:
   - Creates GitHub Release using `softprops/action-gh-release@v2`
   - Uploads firmware binary and documentation
   - Generates release notes with installation instructions
   - Sets release as non-draft and non-prerelease

## Troubleshooting

### The release workflow didn't trigger

- **Check tag format**: Must match `v*.*.*` pattern (e.g., `v1.0.0`, not `1.0.0` or `release-1.0.0`)
- **Verify push**: Run `git ls-remote --tags origin` to see tags on GitHub
- **Check Actions**: Visit the Actions tab to see if the workflow started

### The release workflow failed

1. Check the workflow logs in the Actions tab
2. Common issues:
   - Build failures (check Rust/ESP toolchain setup)
   - Permission issues (workflow needs `contents: write`)
   - Network issues downloading dependencies

### I created a tag by mistake

```bash
# Delete the local tag
git tag -d v1.0.0

# Delete the remote tag (if already pushed)
git push origin --delete v1.0.0
```

Note: If the release was already created, you'll need to manually delete it from the Releases page.

## Examples

### Creating your first release (v1.0.0)

```bash
git checkout main
git pull origin main
git tag -a v1.0.0 -m "First stable release with web UI and dual-core architecture"
git push origin v1.0.0
```

### Creating a patch release (v1.0.1)

```bash
git checkout main
git pull origin main
git tag -a v1.0.1 -m "Fix WiFi reconnection issue"
git push origin v1.0.1
```

### Creating a minor release (v1.1.0)

```bash
git checkout main
git pull origin main
git tag -a v1.1.0 -m "Add support for multiple encoder configurations"
git push origin v1.1.0
```

## Next Steps

Once you've created your first release:

1. Users can download pre-built firmware from the Releases page
2. No need to build from source - just flash the binary
3. Update README.md to reference the latest release
4. Consider adding release badges to README.md

## Additional Resources

- [GitHub Releases Documentation](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [Semantic Versioning](https://semver.org/)
- [Git Tagging](https://git-scm.com/book/en/v2/Git-Basics-Tagging)
