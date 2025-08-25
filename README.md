# Bonding Curve Token Launch Program

Một Solana program được xây dựng bằng Pinocchio framework để tạo và quản lý token với bonding curve pricing mechanism.

## Tính năng

- **Tạo Token với Bonding Curve**: Khởi tạo token mới với pricing curve tùy chỉnh
- **Mua Token**: Mua token từ bonding curve với giá được tính toán động
- **Bán Token**: Bán token về bonding curve và nhận SOL
- **Nhiều loại Curve**: Hỗ trợ Linear, Exponential, và Logarithmic pricing curves
- **Fee System**: Hệ thống phí linh hoạt với fee recipient
- **Slippage Protection**: Bảo vệ người dùng khỏi slippage quá lớn

## Cấu trúc Program

### Instructions

1. **Initialize** (Discriminator: 0)
   - Khởi tạo bonding curve mới với token mint
   - Thiết lập pricing parameters và fee structure

2. **BuyTokens** (Discriminator: 1)
   - Mua token từ bonding curve
   - Tự động tính giá dựa trên curve và supply hiện tại

3. **SellTokens** (Discriminator: 2)
   - Bán token về bonding curve
   - Burn token và trả SOL cho seller

### State

- **BondingCurve**: Account chứa thông tin curve state
  - Authority, token mint, reserves
  - Curve parameters (type, base price, slope)
  - Fee configuration

## Pricing Curves

### Linear Curve (Type 0)
```
price = base_price + (slope * supply / 1e9)
```

### Exponential Curve (Type 1)
```
price = base_price * (1 + slope/1e9)^supply
```

### Logarithmic Curve (Type 2)
```
price = base_price * log(1 + slope * supply / 1000)
```

## Cách sử dụng

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
