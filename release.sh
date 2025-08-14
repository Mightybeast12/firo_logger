#!/bin/bash

# Release script for firo_logger with comprehensive error handling
# Usage: ./release.sh [patch|minor|major|--dry-run|--help]

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
    ./release.sh                    Interactive mode (asks for version increment)
    ./release.sh patch              Increment patch version (1.0.0 → 1.0.1)
    ./release.sh minor              Increment minor version (1.0.0 → 1.1.0)
    ./release.sh major              Increment major version (1.0.0 → 2.0.0)
    ./release.sh --dry-run          Preview changes without executing
    ./release.sh --help             Show this help message

What this script does:
1. ✅ Runs comprehensive pre-flight checks
2. 📝 Updates version in Cargo.toml
3. 📋 Updates CHANGELOG.md
4. 🧪 Runs full test suite
5. 📦 Creates git commit and tag
6. 🚀 Pushes to remote repository

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
        if [ -n "${NEW_VERSION:-}" ]; then
            git tag -d "v$NEW_VERSION" 2>/dev/null || true
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

# Function to increment version
increment_version() {
    local version=$1
    local increment_type=$2

    local major minor patch
    IFS='.' read -r major minor patch <<< "$version"

    case $increment_type in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "${major}.$((minor + 1)).0"
            ;;
        patch)
            echo "${major}.${minor}.$((patch + 1))"
            ;;
        *)
            print_error "Invalid increment type: $increment_type"
            exit 1
            ;;
    esac
}

# Function to update version in Cargo.toml
update_cargo_version() {
    local new_version=$1

    if [ "$DRY_RUN" = "true" ]; then
        print_info "Would update Cargo.toml version to $new_version"
        return
    fi

    if command_exists sed; then
        # Use different sed syntax for macOS vs Linux
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/^version = \"[^\"]*\"/version = \"$new_version\"/" "$CARGO_TOML"
        else
            sed -i "s/^version = \"[^\"]*\"/version = \"$new_version\"/" "$CARGO_TOML"
        fi
    else
        print_error "sed command not found"
        exit 1
    fi
}

# Function to update CHANGELOG.md
update_changelog() {
    local new_version=$1
    local current_date=$(date '+%Y-%m-%d')

    if [ "$DRY_RUN" = "true" ]; then
        print_info "Would update CHANGELOG.md for version $new_version"
        return
    fi

    if [ ! -f "$CHANGELOG" ]; then
        print_warning "CHANGELOG.md not found, skipping changelog update"
        return
    fi

    # Replace "## [Unreleased]" with "## [Unreleased]\n\n## [$new_version] - $current_date"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/## \[Unreleased\]/## [Unreleased]\
\
## [$new_version] - $current_date/" "$CHANGELOG"
    else
        sed -i "s/## \[Unreleased\]/## [Unreleased]\n\n## [$new_version] - $current_date/" "$CHANGELOG"
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

    # Check for uncommitted changes (skip in dry-run mode)
    if ! git diff-index --quiet HEAD --; then
        if [ "$DRY_RUN" = "false" ]; then
            print_error "Uncommitted changes detected"
            print_info "Fix: git add . && git commit -m 'your message' OR git stash"
            git status --short
            exit 1
        else
            print_warning "Uncommitted changes detected (allowed in dry-run mode)"
            git status --short
        fi
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
        if [ "$(git rev-parse HEAD)" != "$(git rev-parse "origin/$MAIN_BRANCH")" ]; then
            print_error "Local branch is not up to date with origin/$MAIN_BRANCH"
            print_info "Fix: git pull origin $MAIN_BRANCH"
            exit 1
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

# Function to check if tag already exists
check_existing_tag() {
    local version=$1
    local tag="v$version"

    if git tag -l "$tag" | grep -q .; then
        print_error "Tag $tag already exists!"
        print_info "Existing tags:"
        git tag -l | tail -5
        exit 1
    fi
}

# Function to get increment type interactively
get_increment_type_interactive() {
    local current_version=$1

    printf "\n"
    printf "${CYAN}🚀 Release Script for firo_logger${NC}\n"
    printf "${BLUE}Current version: $current_version${NC}\n"
    printf "\n"
    printf "How would you like to increment the version?\n"
    printf "1) Patch ($current_version → $(increment_version "$current_version" "patch")) - Bug fixes\n"
    printf "2) Minor ($current_version → $(increment_version "$current_version" "minor")) - New features\n"
    printf "3) Major ($current_version → $(increment_version "$current_version" "major")) - Breaking changes\n"
    printf "4) Cancel\n"
    printf "\n"

    while true; do
        printf "Select (1-4): "
        read choice
        case $choice in
            1) echo "patch"; break ;;
            2) echo "minor"; break ;;
            3) echo "major"; break ;;
            4) print_info "Release cancelled"; exit 0 ;;
            *) printf "Please select 1-4\n" ;;
        esac
    done
}

# Function to show release summary and confirm
confirm_release() {
    local current_version=$1
    local new_version=$2
    local increment_type=$3

    echo
    echo -e "${CYAN}📋 Release Summary:${NC}"
    echo "- Current version: $current_version"
    echo "- New version: $new_version ($increment_type increment)"
    echo "- Files to update: $CARGO_TOML, $CHANGELOG"
    echo "- Tag to create: v$new_version"
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

# Function to create release commit and tag
create_release() {
    local new_version=$1
    local tag="v$new_version"

    if [ "$DRY_RUN" = "true" ]; then
        print_info "Would create commit: chore: release $tag"
        print_info "Would create tag: $tag"
        print_info "Would push to origin/$MAIN_BRANCH"
        return
    fi

    print_step "Creating release commit..."
    git add "$CARGO_TOML" "$CHANGELOG"
    git commit -m "chore: release $tag"

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
    local increment_type=""

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
            patch|minor|major)
                increment_type="$1"
                shift
                ;;
            *)
                print_error "Unknown argument: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    # Store current version
    local current_version=$(get_current_version)

    # Run pre-flight checks
    run_preflight_checks

    # Get increment type if not provided
    if [ -z "$increment_type" ]; then
        increment_type=$(get_increment_type_interactive "$current_version")
    fi

    # Calculate new version
    local new_version=$(increment_version "$current_version" "$increment_type")
    NEW_VERSION="$new_version"  # Store for cleanup function

    # Check if tag already exists
    check_existing_tag "$new_version"

    # Show summary and confirm
    confirm_release "$current_version" "$new_version" "$increment_type"

    # Update version files BEFORE running tests
    print_step "Updating version files..."
    update_cargo_version "$new_version"
    update_changelog "$new_version"

    # Run tests with updated version
    run_tests

    # Create release
    create_release "$new_version"
}

# Run main function with all arguments
main "$@"
