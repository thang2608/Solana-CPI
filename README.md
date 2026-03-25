# 🏦 Solana Week 5: Cross-Program Invocation (CPI)

Chào mừng đến với dự án Tuần 5! Trong tuần này, chúng ta khám phá một trong những tính năng mạnh mẽ và phức tạp nhất của Solana Smart Contract: **Cross-Program Invocation (CPI)** - khả năng một chương trình (Program) gọi và tương tác trực tiếp với một chương trình khác trên chuỗi.

## 🌟 Tổng quan dự án

Dự án mô phỏng một hệ thống tài chính phi tập trung (DeFi) bao gồm 2 chương trình hoạt động độc lập nhưng liên kết chặt chẽ với nhau:

1. **`bank-app` (Ngân hàng):** Nơi người dùng thực hiện các giao dịch cơ bản như `deposit` (gửi tiền), `withdraw` (rút tiền).
2. **`staking-app` (Quỹ Staking):** Một chương trình riêng biệt chuyên quản lý logic trả lãi (APR 5%/năm) và khóa token.

**Tính năng cốt lõi (The Magic):** Thay vì người dùng phải tự tay rút tiền từ Ngân hàng rồi đem qua Quỹ Staking để gửi, `bank-app` sẽ đại diện cho người dùng, sử dụng **CPI** để gọi trực tiếp sang `staking-app`. Ngân hàng sẽ dùng quyền ủy quyền của PDA (Program Derived Address) để ký tên các giao dịch này một cách bảo mật.

## 🧠 Kiến thức trọng tâm đã áp dụng

- **Cross-Program Invocation (CPI):** Cách thiết lập `CpiContext` để gọi hàm từ program khác.
- **PDA Signatures (`invoke_signed`):** Cách một Smart Contract tự động ký tên vào giao dịch (ký thay người dùng) bằng `seeds` và `bump`.
- **Multi-Program Workspace:** Quản lý, cấu hình và test nhiều Anchor Programs trong cùng một dự án.
- **SPL Token Integration:** Xử lý việc nạp/rút/stake cả đồng SOL gốc và các Token tùy chỉnh (Custom SPL Tokens) an toàn qua các `TokenAccount` và `Mint`.

## 📂 Cấu trúc thư mục

```text
05 - Cross Program Invocation (CPI)/
├── programs/
│   ├── bank-app/       # Smart Contract của Ngân hàng
│   └── staking-app/    # Smart Contract của Quỹ Staking (nhận CPI)
├── tests/
│   └── bank-app.ts     # Kịch bản test toàn bộ luồng đi của dòng tiền
├── Anchor.toml         # Cấu hình địa chỉ của cả 2 programs
└── README.md           # Bạn đang đọc nó đây!
```
