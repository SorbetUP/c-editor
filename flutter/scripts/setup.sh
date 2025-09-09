#!/bin/bash

# Flutter setup and verification script

set -e

echo "🔧 Flutter App Container Setup"
echo "==============================="

# Check if Flutter is installed
if ! command -v flutter &> /dev/null; then
    echo "❌ Flutter not found. Installing with Homebrew..."
    brew install --cask flutter
fi

# Check Flutter installation
echo "✅ Checking Flutter installation..."
flutter --version

# Run Flutter doctor
echo "🏥 Running Flutter doctor..."
flutter doctor

# Navigate to flutter directory if not already there
if [[ ! -f "pubspec.yaml" ]]; then
    echo "📁 Navigating to Flutter project directory..."
    cd flutter
fi

# Get dependencies
echo "📦 Installing dependencies..."
flutter pub get

# Generate code
echo "🔄 Generating code..."
flutter packages pub run build_runner build

# Check for analysis issues
echo "🔍 Running static analysis..."
flutter analyze

# Run tests if any exist
if [[ -d "test" ]]; then
    echo "🧪 Running tests..."
    flutter test
fi

echo "✅ Setup complete!"
echo "📱 To run the app:"
echo "   flutter run -d chrome (for web)"
echo "   flutter run -d macos (for macOS)"
echo "   flutter run (for default device)"