#!/bin/bash

# Release script for firo_logger - reads version from Cargo.toml
# Usage: ./release.sh [--dry-run|--help]

set -e  # Exit on any error
set -u  # Exit on undefined variables
set -o pipefail  # Exit on pipe failures

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
MAIN_BRANCH="main"
CARGO_TOML="Cargo.toml"
CHANGELOG="CHANGELOG.md"
DRY_RUN=false

# Function to print colored output
print_error() {
    echo -e "${RED}❌ Error: $1${NC}" >&2
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_step() {
    echo -e "${CYAN}🔄 $1${NC}"
}

# Function to show usage
show_help() {
    cat << EOF
🚀 Release Script for firo_logger

Usage:
    ./release.sh                    Release using version from Cargo.toml
    ./release.sh --dry-run          Preview changes without executing
    ./release.sh --help             Show this help message

What this script does:
1. ✅ Runs comprehensive pre-flight checks
2. 📝 Reads version from Cargo.toml
3. 📋 Updates CHANGELOG.md with current date
4. 💾 Commits changes to Cargo.toml and CHANGELOG.md
5. 🧪 Runs full test suite
6. 📦 Creates git tag
7. 🚀 Pushes to remote repository

Important:
- Update the version in Cargo.toml BEFORE running this script
- The script will commit any uncommitted Cargo.toml changes

Safety features:
- Validates git repository state
- Ensures tests pass before releasing
- Checks for existing tags
- Allows rollback if issues occur
- Confirms actions before pushing

EOF
}

# Function to cleanup on error
cleanup_on_error() {
    local exit_code=$?
    if [ $exit_code -ne 0 ] && [ "$DRY_RUN" = "false" ]; then
        print_warning "Release failed! Cleaning up..."

        # Check if we made any commits that need reverting
        if git log --oneline -1 | grep -q "chore: release v"; then
            print_info "Reverting release commit..."
            git reset --hard HEAD~1 2>/dev/null || true
        else
            # If no commit was made but files were modified, revert them
            if git diff --quiet HEAD -- "$CARGO_TOML" "$CHANGELOG" 2>/dev/null; then
                : # No changes to revert
            else
                print_info "Reverting file changes..."
                git checkout HEAD -- "$CARGO_TOML" "$CHANGELOG" 2>/dev/null || true
            fi
        fi

        # Remove any tags we might have created
        if [ -n "${VERSION:-}" ]; then
            git tag -d "v$VERSION" 2>/dev/null || true
        fi

        print_error "Release aborted. Repository state restored."
    fi
    exit $exit_code
}

# Set up error trap
trap cleanup_on_error EXIT

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to get current version from Cargo.toml
get_current_version() {
    if [ ! -f "$CARGO_TOML" ]; then
        print_error "Cargo.toml not found!"
        exit 1
    fi

    grep '^version' "$CARGO_TOML" | cut -d'"' -f2 | head -1
}

# Function to update CHANGELOG.md
update_changelog() {
    local version=$1
    local current_date=$(date '+%Y-%m-%d')

    if [ "$DRY_RUN" = "true" ]; then
        print_info "Would update CHANGELOG.md for version $version"
        return
    fi

    if [ ! -f "$CHANGELOG" ]; then
        print_warning "CHANGELOG.md not found, skipping changelog update"
        return
    fi

    # Replace "## [Unreleased]" with "## [Unreleased]\n\n## [$version] - $current_date"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/## \[Unreleased\]/## [Unreleased]\
\
## [$version] - $current_date/" "$CHANGELOG"
    else
        sed -i "s/## \[Unreleased\]/## [Unreleased]\n\n## [$version] - $current_date/" "$CHANGELOG"
    fi
}

# Function to run comprehensive pre-flight checks
run_preflight_checks() {
    print_step "Running pre-flight checks..."

    # Check if git is installed
    if ! command_exists git; then
        print_error "git is not installed or not in PATH"
        exit 1
    fi

    # Check if we're in a git repository
    if ! git rev-parse --git-dir >/dev/null 2>&1; then
        print_error "Not in a git repository"
        exit 1
    fi

    # Check if cargo is installed
    if ! command_exists cargo; then
        print_error "cargo is not installed or not in PATH"
        exit 1
    fi

    # Check if we're on the main branch
    local current_branch=$(git branch --show-current)
    if [ "$current_branch" != "$MAIN_BRANCH" ]; then
        print_error "Not on $MAIN_BRANCH branch (currently on: $current_branch)"
        print_info "Fix: git checkout $MAIN_BRANCH"
        exit 1
    fi

    # Check if we can reach the remote (skip in dry-run mode)
    if [ "$DRY_RUN" = "false" ]; then
        if ! git ls-remote origin >/dev/null 2>&1; then
            print_error "Cannot reach remote repository"
            print_info "Fix: Check your internet connection and git remote configuration"
            exit 1
        fi

        # Check if we're up to date with remote
        git fetch origin "$MAIN_BRANCH" --quiet
        local local_commit=$(git rev-parse HEAD)
        local remote_commit=$(git rev-parse "origin/$MAIN_BRANCH")

        if [ "$local_commit" != "$remote_commit" ]; then
            # Check if local is behind remote (need to pull)
            if git merge-base --is-ancestor "$local_commit" "$remote_commit"; then
                print_error "Local branch is behind origin/$MAIN_BRANCH"
                print_info "Fix: git pull origin $MAIN_BRANCH"
                exit 1
            fi
            # Local is ahead of remote - will push during release
            print_info "Local branch has unpushed commits - will push during release"
        fi
    else
        print_info "Skipping remote checks in dry-run mode"
    fi

    print_success "Pre-flight checks passed"
}

# Function to run tests
run_tests() {
    print_step "Running test suite..."

    if [ "$DRY_RUN" = "true" ]; then
        print_info "Would run: cargo test --all-features"
        return
    fi

    if ! ./test-local.sh; then
        print_error "Tests failed! Fix issues before releasing."
        print_info "Run './test-local.sh' to see detailed test results"
        exit 1
    fi

    print_success "All tests passed"
}

# Function to handle existing tags by removing them
handle_existing_tag() {
    local version=$1
    local tag="v$version"
    local tag_exists_locally=false
    local tag_exists_remotely=false

    # Check if tag exists locally
    if git tag -l "$tag" | grep -q .; then
        tag_exists_locally=true
    fi

    # Check if tag exists remotely (skip in dry-run mode)
    if [ "$DRY_RUN" = "false" ]; then
        if git ls-remote --tags origin | grep -q "refs/tags/$tag"; then
            tag_exists_remotely=true
        fi
    fi

    # If tag doesn't exist anywhere, we're done
    if [ "$tag_exists_locally" = "false" ] && [ "$tag_exists_remotely" = "false" ]; then
        return
    fi

    # Tag exists - handle it
    print_warning "Tag $tag already exists - removing stale tag..."

    # Remove local tag
    if [ "$tag_exists_locally" = "true" ]; then
        if [ "$DRY_RUN" = "true" ]; then
            print_info "Would remove local tag: $tag"
        else
            print_step "Removing local tag $tag"
            git tag -d "$tag"
            print_success "Local tag removed"
        fi
    fi

    # Remove remote tag
    if [ "$tag_exists_remotely" = "true" ]; then
        if [ "$DRY_RUN" = "true" ]; then
            print_info "Would remove remote tag: $tag"
        else
            print_step "Removing remote tag $tag"
            if git push origin ":refs/tags/$tag" 2>/dev/null; then
                print_success "Remote tag removed"
            else
                print_warning "Failed to remove remote tag (might not have permissions)"
                print_info "You may need to manually remove it with: git push origin :refs/tags/$tag"
            fi
        fi
    fi

    print_success "Stale tag cleaned up, continuing with release"
}

# Function to show release summary and confirm
confirm_release() {
    local version=$1

    echo
    echo -e "${CYAN}📋 Release Summary:${NC}"
    echo "- Version: $version (from Cargo.toml)"
    echo "- Files to update: $CHANGELOG"
    echo "- Files to commit: $CARGO_TOML, $CHANGELOG"
    echo "- Tag to create: v$version"
    echo "- Branch: $MAIN_BRANCH"
    echo

    if [ "$DRY_RUN" = "true" ]; then
        print_info "DRY RUN MODE - No changes will be made"
        return
    fi

    print_warning "This will create a commit and push to origin/$MAIN_BRANCH"
    echo

    while true; do
        read -p "Continue with release? (y/N): " confirm
        case $confirm in
            [Yy]* ) break ;;
            [Nn]* | "" ) print_info "Release cancelled"; exit 0 ;;
            * ) echo "Please answer yes or no" ;;
        esac
    done
}

# Function to commit changes
commit_changes() {
    local version=$1

    if [ "$DRY_RUN" = "true" ]; then
        print_info "Would add and commit: $CARGO_TOML $CHANGELOG"
        return
    fi

    print_step "Committing version changes..."

    # Add both files (in case Cargo.toml has uncommitted changes)
    git add "$CARGO_TOML" "$CHANGELOG"

    # Check if there are changes to commit
    if git diff --cached --quiet; then
        print_warning "No changes to commit"
    else
        git commit -m "chore: release v$version"
        print_success "Changes committed"
    fi
}

# Function to create tag and push
create_tag_and_push() {
    local version=$1
    local tag="v$version"

    if [ "$DRY_RUN" = "true" ]; then
        print_info "Would create tag: $tag"
        print_info "Would push to origin/$MAIN_BRANCH"
        return
    fi

    print_step "Creating git tag..."
    git tag -a "$tag" -m "Release $tag"

    print_step "Pushing to remote..."
    git push origin "$MAIN_BRANCH"
    git push origin "$tag"

    print_success "Release $tag completed successfully!"
    echo
    print_info "GitHub Actions will now:"
    print_info "- Run CI checks"
    print_info "- Publish to crates.io"
    print_info "- Create GitHub release"
    echo
    print_info "View release: https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\([^/]*\/[^/]*\).*/\1/' | sed 's/\.git$//')/releases/tag/$tag"
}

# Main function
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --help|-h)
                show_help
                exit 0
                ;;
            --dry-run)
                DRY_RUN=true
                print_info "Running in DRY RUN mode"
                shift
                ;;
            *)
                print_error "Unknown argument: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    # Get version from Cargo.toml
    local version=$(get_current_version)
    VERSION="$version"  # Store for cleanup function

    print_step "Preparing release for firo_logger v$version"

    # Run pre-flight checks
    run_preflight_checks

    # Handle existing tag (remove if exists)
    handle_existing_tag "$version"

    # Show summary and confirm
    confirm_release "$version"

    # Update changelog
    print_step "Updating CHANGELOG.md..."
    update_changelog "$version"

    # Commit changes (both Cargo.toml and CHANGELOG.md)
    commit_changes "$version"

    # Run tests with updated and committed version
    run_tests

    # Create tag and push
    create_tag_and_push "$version"
}

# Run main function with all arguments
main "$@"
