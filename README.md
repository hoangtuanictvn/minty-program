# Bonding Curve Token Launch Program

A Solana program built with the Pinocchio framework to create and manage tokens using a bonding curve pricing mechanism.

## Features

- **Create Token with Bonding Curve**: Initialize a new token with customizable pricing curve
- **Buy Tokens**: Purchase tokens from the bonding curve with dynamically calculated price
- **Sell Tokens**: Sell tokens back to the bonding curve and receive SOL
- **Fee System**: Flexible fee mechanism with a fee recipient
- **Slippage Protection**: Protect users from excessive slippage

## Program Structure

### Instructions

1. **Initialize** (Discriminator: 0)
   - Initialize a new bonding curve for a token mint
   - Configure pricing parameters and fee structure

2. **BuyTokens** (Discriminator: 1)
   - Buy tokens from the bonding curve
   - Price is computed automatically based on the curve and current supply

3. **SellTokens** (Discriminator: 2)
   - Sell tokens back to the bonding curve
   - Burn tokens and transfer SOL to the seller

### State

- **BondingCurve**: Account storing curve state
  - Authority, token mint, reserves
  - Curve parameters (type, base price, slope)
  - Fee configuration

## Usage

### 1. Build Program

```bash
cargo build
```

### 2. Deploy Program

```bash
solana program deploy target/deploy/x_token.so
```

## Dependencies

- `pinocchio`: Core framework
- `pinocchio-token`: Token program interactions
- `pinocchio-system`: System program interactions
- `bytemuck`: Zero-copy serialization

## License

MIT License