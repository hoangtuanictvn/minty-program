#!/bin/bash

# Script để generate client code từ codama
# Chạy từ thư mục x-token-program

echo "🚀 Starting client code generation with Codama..."

# Kiểm tra xem có đang ở đúng thư mục không
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from x-token-program directory"
    echo "Current directory: $(pwd)"
    exit 1
fi

# Kiểm tra xem có thư mục codama không
if [ ! -d "../codama" ]; then
    echo "❌ Error: codama directory not found. Please make sure it exists at ../codama"
    exit 1
fi

# Chuyển đến thư mục codama
echo "📁 Switching to codama directory..."
cd ../codama

# Kiểm tra xem có package.json không
if [ ! -f "package.json" ]; then
    echo "❌ Error: package.json not found in codama directory"
    exit 1
fi

# Kiểm tra xem có bun không
if ! command -v bun &> /dev/null; then
    echo "❌ Error: bun is not installed. Please install bun first:"
    echo "   curl -fsSL https://bun.sh/install | bash"
    exit 1
fi

# Kiểm tra dependencies
echo "📦 Checking dependencies..."
if [ ! -d "node_modules" ]; then
    echo "📥 Installing dependencies..."
    bun install
fi

# Generate client code
echo "🔧 Generating client code..."
bun run generate-client.ts

# Kiểm tra kết quả
if [ $? -eq 0 ]; then
    echo "✅ Client code generated successfully!"
    echo "📁 Generated files are in: $(pwd)/clients/xToken/"
    
    # Copy generated files to minty project
    echo "📋 Copying generated files to minty project..."
    if [ -d "../../minty/src/lib/xToken" ]; then
        cp -r clients/xToken/* ../../minty/src/lib/xToken/
        echo "✅ Files copied to minty project successfully!"
    else
        echo "⚠️  Warning: minty project directory not found at ../../minty/src/lib/xToken"
        echo "   Please copy the generated files manually from: $(pwd)/clients/xToken/"
    fi
else
    echo "❌ Error: Failed to generate client code"
    exit 1
fi

echo "🎉 Client code generation completed!"