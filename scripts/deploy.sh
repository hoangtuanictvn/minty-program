#!/bin/bash

# Bonding Curve Program Deployment Script

set -e

echo "🚀 Deploying Bonding Curve Program..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if Solana CLI is installed
if ! command -v solana &> /dev/null; then
    echo -e "${RED}❌ Solana CLI is not installed. Please install it first.${NC}"
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo is not installed. Please install Rust first.${NC}"
    exit 1
fi

# Set Solana config (default to devnet)
NETWORK=${1:-devnet}
echo -e "${BLUE}📡 Setting Solana config to $NETWORK...${NC}"
solana config set --url $NETWORK

# Check balance
BALANCE=$(solana balance --lamports | grep -o '[0-9]*' | head -1)
if [ -z "$BALANCE" ] || [ "$BALANCE" -lt 1000000000 ]; then
    echo -e "${YELLOW}⚠️  Low balance detected. You might need more SOL for deployment.${NC}"
    echo -e "${YELLOW}💰 Current balance: $(solana balance)${NC}"
    
    if [ "$NETWORK" = "devnet" ]; then
        echo -e "${BLUE}🪂 Requesting airdrop...${NC}"
        solana airdrop 2
    fi
fi

# Build the program
echo -e "${BLUE}🔨 Building program...${NC}"
cargo build-sbf

# Check if build was successful
if [ ! -f "target/deploy/x_token.so" ]; then
    echo -e "${RED}❌ Build failed. Program binary not found.${NC}"
    exit 1
fi

# Deploy the program
echo -e "${BLUE}🚀 Deploying program...${NC}"
DEPLOY_OUTPUT=$(solana program deploy target/deploy/x_token.so)
PROGRAM_ID=$(echo "$DEPLOY_OUTPUT" | grep -o 'Program Id: [A-Za-z0-9]*' | cut -d' ' -f3)

if [ "$PROGRAM_ID" = "null" ] || [ -z "$PROGRAM_ID" ]; then
    echo -e "${RED}❌ Deployment failed.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Program deployed successfully!${NC}"
echo -e "${GREEN}📋 Program ID: $PROGRAM_ID${NC}"

# Update program ID in client
if [ -f "client/bonding_curve_client.ts" ]; then
    echo -e "${BLUE}🔄 Updating program ID in client...${NC}"
    
    # Create backup
    cp client/bonding_curve_client.ts client/bonding_curve_client.ts.bak
    
    # Update program ID
    sed -i.tmp "s/const PROGRAM_ID = new PublicKey('.*');/const PROGRAM_ID = new PublicKey('$PROGRAM_ID');/" client/bonding_curve_client.ts
    rm client/bonding_curve_client.ts.tmp
    
    echo -e "${GREEN}✅ Client updated with new program ID${NC}"
else
    echo -e "${YELLOW}⚠️  Client file not found. Please manually update the program ID.${NC}"
fi

# Update program ID in lib.rs
if [ -f "src/lib.rs" ]; then
    echo -e "${BLUE}🔄 Updating program ID in lib.rs...${NC}"
    
    # Create backup
    cp src/lib.rs src/lib.rs.bak
    
    # Update program ID (this is more complex due to the declare_id! macro)
    echo -e "${YELLOW}⚠️  Please manually update the program ID in src/lib.rs:${NC}"
    echo -e "${YELLOW}   pinocchio_pubkey::declare_id!(\"$PROGRAM_ID\");${NC}"
fi

# Save deployment info
echo -e "${BLUE}💾 Saving deployment info...${NC}"
cat > deployment.json << EOF
{
  "network": "$NETWORK",
  "programId": "$PROGRAM_ID",
  "deployedAt": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "deployer": "$(solana address)"
}
EOF

echo -e "${GREEN}✅ Deployment info saved to deployment.json${NC}"

echo -e "${GREEN}✅ Deployment info saved to deployment.json${NC}"

# Show next steps
echo -e "${BLUE}📋 Next Steps:${NC}"
echo -e "${BLUE}1. Update program ID in src/lib.rs if not done automatically${NC}"
echo -e "${BLUE}2. Rebuild and redeploy if you updated lib.rs${NC}"
echo -e "${BLUE}3. Test the client: cd client && npm install && npm run test${NC}"
echo -e "${BLUE}4. Fund the test account that will be created${NC}"

echo -e "${GREEN}🎉 Deployment completed!${NC}"
