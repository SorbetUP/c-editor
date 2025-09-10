#!/bin/bash
set -e

echo "=== Triggering GitHub Pages Build ==="

# Check if gh CLI is available
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is not installed"
    echo "Install with: brew install gh"
    echo "Then run: gh auth login"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "❌ Not authenticated with GitHub CLI"
    echo "Run: gh auth login"
    exit 1
fi

echo "🔍 Checking repository status..."
REPO_STATUS=$(git status --porcelain)
if [ -n "$REPO_STATUS" ]; then
    echo "⚠️ Repository has uncommitted changes:"
    git status --short
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "🚀 Triggering GitHub Pages deployment workflow..."

# Trigger the Pages workflow manually
gh workflow run pages.yml

echo "✅ GitHub Pages build triggered!"
echo ""
echo "🔍 You can monitor the build at:"
echo "   https://github.com/SorbetUP/c-editor/actions"
echo ""
echo "📄 Once deployed, your page will be available at:"
echo "   https://SorbetUP.github.io/c-editor/"
echo ""
echo "⏱️  Build typically takes 2-5 minutes to complete."