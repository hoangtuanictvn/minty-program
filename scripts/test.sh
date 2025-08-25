#!/bin/bash

# Bonding Curve Program Test Script

set -e

echo "🧪 Running Bonding Curve Program Tests..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to run Rust tests
run_rust_tests() {
    echo -e "${BLUE}🦀 Running Rust unit tests...${NC}"
    
    if cargo test; then
        echo -e "${GREEN}✅ Rust tests passed!${NC}"
    else
        echo -e "${RED}❌ Rust tests failed!${NC}"
        return 1
    fi
}

# Function to run TypeScript client tests
run_client_tests() {
    echo -e "${BLUE}📜 Running TypeScript client tests...${NC}"
    
    if [ ! -d "client/node_modules" ]; then
        echo -e "${BLUE}📦 Installing client dependencies...${NC}"
        cd client
        npm install
        cd ..
    fi
    
    cd client
    
    # Check if payer keypair exists
    if [ ! -f "payer-keypair.json" ]; then
        echo -e "${YELLOW}⚠️  No payer keypair found. Client test will create one.${NC}"
        echo -e "${YELLOW}💰 Please fund the generated account before running full tests.${NC}"
    fi
    
    # Run client tests
    if npm run test; then
        echo -e "${GREEN}✅ Client tests completed!${NC}"
    else
        echo -e "${RED}❌ Client tests failed!${NC}"
        cd ..
        return 1
    fi
    
    cd ..
}

# Function to check program deployment
check_deployment() {
    echo -e "${BLUE}🔍 Checking program deployment...${NC}"
    
    if [ -f "deployment.json" ]; then
        PROGRAM_ID=$(jq -r '.programId' deployment.json)
        NETWORK=$(jq -r '.network' deployment.json)
        
        echo -e "${GREEN}📋 Found deployment:${NC}"
        echo -e "${GREEN}   Program ID: $PROGRAM_ID${NC}"
        echo -e "${GREEN}   Network: $NETWORK${NC}"
        
        # Check if program exists on-chain
        if solana account $PROGRAM_ID > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Program found on-chain${NC}"
        else
            echo -e "${YELLOW}⚠️  Program not found on-chain. May need to deploy first.${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  No deployment.json found. Run deploy.sh first.${NC}"
    fi
}

# Function to run integration tests
run_integration_tests() {
    echo -e "${BLUE}🔗 Running integration tests...${NC}"
    
    # This would run more comprehensive tests that interact with the deployed program
    # For now, we'll just run the existing tests
    echo -e "${BLUE}ℹ️  Integration tests would go here${NC}"
    echo -e "${BLUE}ℹ️  Currently running unit tests and client tests${NC}"
}

# Main execution
main() {
    local test_type=${1:-all}
    
    case $test_type in
        "rust")
            run_rust_tests
            ;;
        "client")
            check_deployment
            run_client_tests
            ;;
        "integration")
            check_deployment
            run_integration_tests
            ;;
        "all")
            echo -e "${BLUE}🚀 Running all tests...${NC}"
            run_rust_tests
            check_deployment
            run_client_tests
            run_integration_tests
            ;;
        *)
            echo -e "${RED}❌ Unknown test type: $test_type${NC}"
            echo -e "${BLUE}Usage: $0 [rust|client|integration|all]${NC}"
            exit 1
            ;;
    esac
    
    echo -e "${GREEN}🎉 Test suite completed!${NC}"
}

# Show usage if help requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo -e "${BLUE}Bonding Curve Program Test Script${NC}"
    echo ""
    echo -e "${BLUE}Usage:${NC}"
    echo -e "  $0 [test_type]"
    echo ""
    echo -e "${BLUE}Test Types:${NC}"
    echo -e "  rust        - Run Rust unit tests only"
    echo -e "  client      - Run TypeScript client tests only"
    echo -e "  integration - Run integration tests"
    echo -e "  all         - Run all tests (default)"
    echo ""
    echo -e "${BLUE}Examples:${NC}"
    echo -e "  $0          # Run all tests"
    echo -e "  $0 rust     # Run only Rust tests"
    echo -e "  $0 client   # Run only client tests"
    exit 0
fi

# Run main function
main "$1"
