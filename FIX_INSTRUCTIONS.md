# Chỗ cần sửa và cách sửa (cụ thể theo file)

## 1) `src/main.rs`

### Vấn đề
- File đang là pseudo-code (`...`, biến chưa khai báo, thiếu import), nên project không build được.

### Sửa như nào
- Dùng entrypoint tối thiểu, gọi đúng API đã có (`start_core_engine`) thay vì tự ghép thread/channel thủ công chưa hoàn chỉnh.
- Sau đó mở rộng dần gateway/execution theo pipeline chuẩn.

## 2) `src/core.rs`

### Vấn đề
- `add_order` tạo `order` nhưng không `push` vào `orders_arena` và không cập nhật `id_map`.
- `cancel_order` chỉ lookup `buy_limits`, hủy lệnh sell sai.
- Có phép cast sai kiểu (`shares as U64`) gây rủi ro compile/runtime logic.

### Sửa như nào
- Thêm trường `is_buy` vào `Order` để biết order nằm ở book nào khi cancel.
- Trong `add_order`:
  - cập nhật linked-list level,
  - `limit.total_volume += shares`,
  - `self.orders_arena.push(order)`,
  - `self.id_map.insert(id, idx)`.
- Trong `cancel_order`:
  - tra side theo `order.is_buy`,
  - unlink chính xác `prev/next`,
  - trừ `total_volume` và xóa level khi rỗng.
- Trong `execute_match`:
  - tách nhánh buy/sell rõ ràng,
  - match với best level đối ứng,
  - cập nhật maker order/limit volume/id_map nhất quán.

## 3) `src/types.rs`

### Vấn đề
- Có import không dùng (`hashbrown::HashMap`) gây warning, làm nhiễu khi theo dõi warning thực sự quan trọng.

### Sửa như nào
- Xóa import thừa để warning sạch hơn.

## 4) Gợi ý bước tiếp theo bạn nên làm ngay

1. Bổ sung `event bus` thống nhất: `start_core_engine` nên publish `EngineEvent::Trades` ra channel riêng, execution manager đọc từ channel đó.
2. Sửa execution side:
   - không gọi cứng `limit_buy` cho mọi trade,
   - map đúng Buy/Sell theo ngữ cảnh chiến lược.
3. Tách market-data command khỏi order command (đừng tạo `resp channel` cho từng tick depth).
4. Thêm unit test cho `OrderBook`:
   - add/cancel,
   - partial fill,
   - nhiều lệnh cùng mức giá (FIFO).
