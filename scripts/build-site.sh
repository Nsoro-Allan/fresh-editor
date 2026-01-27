#!/bin/bash
set -e

# Clean
rm -rf dist

# Build VitePress docs
# It will build to dist/docs because of outDir in config.ts
bun run docs:build

# Copy custom homepage to root of dist
cp homepage/index.html dist/

# Copy all homepage public files (assets, icons, etc.) to dist/public
if [ -d "homepage/public" ]; then
    cp -r homepage/public dist/
fi

# Add .nojekyll to bypass Jekyll on GitHub Pages
touch dist/.nojekyll

echo "Build complete! Output is in dist/"
