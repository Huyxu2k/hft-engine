# Code review & hướng tối ưu (hft-engine)

## 1) Vấn đề blocking hiện tại (không build được)

1. `src/main.rs` đang là pseudo-code: có `...`, thiếu import/type/function (`OrderCommand`, `EngineEvent`, `thread`, `OrderBook`, `start_execution_manager`, `symbol_config`) nên `cargo check` fail ngay.
2. `README.md` mô tả cấu trúc file (`config.rs`, `market.rs`, `strategy.rs`) không khớp với cây thư mục thực tế.

## 2) Vấn đề kiến trúc/độ đúng logic

1. Dòng chảy sự kiện chưa kín: trong `start_core_engine`, lệnh `Add` trả `trades` qua `resp`, nhưng `EngineEvent::Trades` không được publish từ core sang execution manager theo một bus sự kiện thống nhất.
2. `start_execution_manager` luôn gọi `account.limit_buy(...)` cho mọi trade; thiếu phân nhánh buy/sell và thiếu mapping maker/taker -> side thực thi.
3. `run_binance_gateway` dùng `id: 0` cho mọi lệnh market data; điều này làm hỏng semantics hủy/sửa lệnh và khó phân biệt lệnh hệ thống vs lệnh chiến lược.
4. `run_binance_gateway` tạo `resp: crossbeam_channel::unbounded().0` cho mỗi update depth -> tăng allocations không cần thiết.

## 3) Vấn đề hiệu năng (latency/throughput)

1. `core::add_order` khởi tạo `order` nhưng chưa push vào `orders_arena` và chưa ghi `id_map`; vừa sai logic vừa bỏ lỡ mô hình arena/FIFO mong muốn.
2. `cancel_order` chỉ tra `buy_limits` (theo comment “giả sử Buy”) -> hủy lệnh phía sell có nguy cơ sai.
3. Dùng `BTreeMap` cho mỗi thao tác là `O(log M)`; với HFT có thể cân nhắc cấu trúc per-price-level + queue và top-of-book cache để giảm branch/cache-miss.
4. Websocket callback đang `send(...).unwrap()` đồng bộ trong vòng lặp update depth; khi consumer chậm sẽ gây áp lực tail latency.

## 4) Vấn đề an toàn/vận hành

1. Nhiều `unwrap()` ở đường nóng (gateway/connect/event loop), có thể làm process chết đột ngột.
2. `risk::allow` hardcode ngưỡng vị thế `±5`, chưa config-driven theo symbol/account.
3. Chưa thấy cơ chế rate-limit/throttling/circuit-breaker rõ ràng ở execution layer.

## 5) Ưu tiên tối ưu đề xuất (theo thứ tự triển khai)

### P0 (bắt buộc)

1. Làm sạch `main.rs` để biên dịch được: import đầy đủ, bỏ pseudo token `...`, truyền tham số đúng chữ ký hàm.
2. Chuẩn hóa một event pipeline duy nhất:
   - Core nhận `OrderCommand`
   - Core phát `EngineEvent`
   - Execution chỉ đọc `EngineEvent`
3. Sửa `OrderBook::add_order/cancel_order` để đúng dữ liệu:
   - push vào `orders_arena`
   - update `id_map`
   - cancel xử lý đúng cả buy/sell

### P1 (hiệu năng)

1. Thay vì cấp phát channel response mỗi message depth, dùng một fast-path command riêng cho market data (không cần `resp`) hoặc dùng object pool.
2. Giảm parse/convert lặp lại (fixed-point): chuẩn hóa ngay tại ingress, tránh convert qua lại `f64` nếu không cần.
3. Thêm pre-allocation cho vectors/buffers ở đường nóng (trades batch, depth updates).

### P2 (độ tin cậy)

1. Thay `unwrap()` bằng xử lý lỗi có phân loại: retryable vs fatal.
2. Thêm metrics & tracing:
   - queue depth
   - match latency
   - reject rate
   - API error rate
3. Thêm replay/snapshot để recovery sau restart.

## 6) Đề xuất test tối thiểu

1. Unit test cho `OrderBook`:
   - FIFO cùng mức giá
   - partial fill
   - cancel head/middle/tail
2. Integration test cho pipeline command -> event -> execution (mock exchange).
3. Property test cho invariants:
   - tổng volume level = tổng qty order trong level
   - không có order mồ côi trong `id_map`/arena

## 7) Kết luận ngắn

Code hiện đang ở mức prototype/ghi chú ý tưởng. Ưu tiên trước mắt là **đưa project về trạng thái compile + event flow nhất quán + đúng logic book**, sau đó mới tối ưu vi mô cho latency.
