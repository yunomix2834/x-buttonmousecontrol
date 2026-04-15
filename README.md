# xbuttonmousecontrol

Ứng dụng bind nút chuột thành phím bàn phím trên **Windows** và **Linux/X11**, được tổ chức theo hướng **Hexagonal Architecture** để dễ mở rộng và tuân thủ các nguyên tắc **SOLID**.

> Hiện tại nhánh Linux của project đang đi theo hướng **X11**. Nếu bạn đang dùng **Wayland**, chương trình có thể không hoạt động đúng như mong đợi.

## Tính năng hiện có

- Lắng nghe sự kiện nhấn/thả chuột toàn cục
- Map nút chuột thành phím bàn phím
- Hỗ trợ 2 kiểu action:
  - `tap`: nhấn chuột một lần thì bấm phím một lần
  - `hold`: giữ chuột thì giữ phím, thả chuột thì nhả phím

## Yêu cầu môi trường

### 1. Cài Rust

Nếu máy chưa có `cargo`, cài Rust bằng `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Kiểm tra:

```bash
cargo --version
rustc --version
```

### 2. Linux: cài dependency hệ thống

Trên Ubuntu/Debian, cài thêm các gói cần cho X11:

```bash
sudo apt-get update
sudo apt-get install -y \
  pkg-config \
  libx11-dev \
  libxtst-dev \
  libxi-dev \
  libxdo-dev
```

Nếu thiếu các gói này, khi build bạn có thể gặp lỗi kiểu:
- `Package 'xi' not found`
- `xi.pc not found`
- lỗi từ crate `x11`

## Cách chạy project

### Chạy trực tiếp từ source

Tại thư mục root project:

```bash
cargo run -p xbuttonmousecontrol-cli -- config/bindings.toml
```

### Build release

#### Linux

```bash
cargo build --release -p xbuttonmousecontrol-cli
./target/release/xbuttonmousecontrol-cli config/bindings.toml
```

#### Windows

```powershell
cargo build --release -p xbuttonmousecontrol-cli
.\target\release\xbuttonmousecontrol-cli.exe config\bindings.toml
```

## Cấu hình binding

File cấu hình mặc định nằm ở:

```text
config/bindings.toml
```

Ví dụ:

```toml
# action:
# - hold: press mouse down => press key, release mouse => release key
# - tap: click key once when mouse is pressed

[[bindings]]
mouse_button = "back"
key = "space"
action = "tap"

[[bindings]]
mouse_button = "forward"
key = "ctrl"
action = "hold"

[[bindings]]
mouse_button = "middle"
key = "f8"
action = "tap"
```

### Ý nghĩa các field

#### `mouse_button`
Các giá trị đang hỗ trợ:
- `left`
- `right`
- `middle`
- `back`
- `forward`
- alias:
  - `x1`
  - `x2`
  - `mouse4`
  - `mouse5`

#### `key`
Tên phím muốn map tới.

Ví dụ:
- `space`
- `ctrl`
- `shift`
- `alt`
- `enter`
- `tab`
- `esc`
- `f1` ... `f24`
- ký tự đơn như `a`, `b`, `z`, `1`

#### `action`
- `tap`: nhấn chuột xuống thì gửi 1 lần click phím
- `hold`: nhấn chuột xuống thì giữ phím, thả chuột ra thì nhả phím

## Ví dụ sử dụng

### Ví dụ 1: Nút Back thành Space

```toml
[[bindings]]
mouse_button = "back"
key = "space"
action = "tap"
```

Khi bấm nút Back trên chuột, chương trình sẽ gửi phím `Space`.

### Ví dụ 2: Nút Forward thành Ctrl giữ liên tục

```toml
[[bindings]]
mouse_button = "forward"
key = "ctrl"
action = "hold"
```

Khi giữ nút Forward trên chuột, chương trình sẽ giữ `Ctrl`. Khi thả nút chuột, `Ctrl` sẽ được nhả ra.

### Ví dụ 3: Nút giữa chuột thành F8

```toml
[[bindings]]
mouse_button = "middle"
key = "f8"
action = "tap"
```