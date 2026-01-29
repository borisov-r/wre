#!/bin/bash

# Release Helper Script for WRE (Wireless Rotary Encoder)
# This script helps create and push release tags following semantic versioning

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored messages
print_info() {
    echo -e "${BLUE}ℹ ${NC}$1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Function to validate semantic version
validate_version() {
    if [[ ! $1 =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        return 1
    fi
    return 0
}

# Main script
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "   WRE Release Helper"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d ".github/workflows" ]; then
    print_error "This script must be run from the root of the WRE repository"
    exit 1
fi

# Check if git is clean
if [ -n "$(git status --porcelain)" ]; then
    print_warning "You have uncommitted changes:"
    git status --short
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Aborting release"
        exit 0
    fi
fi

# Check current branch
current_branch=$(git branch --show-current)
if [ "$current_branch" != "main" ] && [ "$current_branch" != "master" ]; then
    print_warning "You are on branch '$current_branch', not 'main' or 'master'"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Aborting release"
        exit 0
    fi
fi

# List existing tags
print_info "Existing tags:"
if git tag -l | grep -q "^v"; then
    git tag -l | grep "^v" | sort -V | tail -5
else
    echo "  (no version tags found)"
fi
echo ""

# Get version from user
while true; do
    read -p "Enter version number (e.g., 1.0.0): " version
    
    if validate_version "$version"; then
        tag_name="v$version"
        
        # Check if tag already exists
        if git rev-parse "$tag_name" >/dev/null 2>&1; then
            print_error "Tag $tag_name already exists!"
            echo ""
            continue
        fi
        
        break
    else
        print_error "Invalid version format. Use semantic versioning (e.g., 1.0.0)"
        echo ""
    fi
done

echo ""
print_info "Version: $tag_name"
echo ""

# Get release message from user
read -p "Enter release message (optional, press Enter to skip): " release_message

if [ -z "$release_message" ]; then
    release_message="Release version $version"
fi

echo ""
print_info "Tag message: $release_message"
echo ""

# Confirmation
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Tag:     $tag_name"
echo "  Message: $release_message"
echo "  Branch:  $current_branch"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

read -p "Create and push this release tag? (y/N): " -n 1 -r
echo
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_info "Release cancelled"
    exit 0
fi

# Create and push the tag
print_info "Creating tag $tag_name..."
git tag -a "$tag_name" -m "$release_message"
print_success "Tag created"

echo ""
print_info "Pushing tag to origin..."
if git push origin "$tag_name"; then
    print_success "Tag pushed to GitHub"
else
    print_error "Failed to push tag to GitHub"
    print_warning "The tag was created locally but not pushed."
    print_warning "You can manually push it later with: git push origin $tag_name"
    exit 1
fi

echo ""
print_success "Release process started!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
print_info "Next steps:"
REPO_PATH=$(git remote get-url origin | sed -e 's/.*github.com[:/]//' -e 's/\.git$//')
echo "  1. Go to: https://github.com/$REPO_PATH/actions"
echo "  2. Watch the 'Release' workflow (takes ~10-15 minutes)"
echo "  3. Check releases: https://github.com/$REPO_PATH/releases"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
