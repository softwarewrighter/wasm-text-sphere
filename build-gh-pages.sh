#!/bin/bash
# Build script for GitHub Pages deployment

set -e

echo "Building WASM module..."
wasm-pack build --target web --release --out-dir www/pkg

echo "Preparing docs directory for GitHub Pages..."
rm -rf docs
mkdir -p docs

echo "Copying static files..."
cp www/index.html docs/
cp www/style.css docs/
cp www/favicon.ico docs/ 2>/dev/null || true
cp -r www/pkg docs/

echo "Removing pkg .gitignore (so built files are committed)..."
rm -f docs/pkg/.gitignore

echo "Adding .nojekyll file..."
touch docs/.nojekyll

echo "Build complete! docs/ directory is ready for GitHub Pages."
echo "Commit and push to deploy."
