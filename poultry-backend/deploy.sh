#!/usr/bin/env bg
set -e

echo "🚀 Building poultry-backend for ARM64..."
cargo lambda build --release --arm64 --package poultry-backend

echo "📦 Deploying poultry-backend to AWS Lambda..."
cd "$(dirname "$0")"
cargo lambda deploy poultry-backend

echo "✅ Backend deployed successfully!"
