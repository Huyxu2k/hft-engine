```
src/
├── main.rs          # entrypoint
├── config.rs        # load config
├── market.rs        # websocket + orderbook
├── strategy.rs      # logic giao dịch
├── risk.rs          # kiểm soát rủi ro
├── trader.rs        # đặt / hủy lệnh
├── engine.rs        # nối market → strategy → trader
└── types.rs         # struct + enum dùng chung
```