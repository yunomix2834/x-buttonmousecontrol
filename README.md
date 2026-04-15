# xbuttonmousecontrol

Ứng dụng bind **chuột → phím** và **phím → click chuột** trên **Windows** và **Linux/X11**.

Project được tổ chức theo hướng **Hexagonal Architecture**:
- `core`: domain + ports + runtime
- `config-toml`: adapter đọc cấu hình TOML
- `platform-rdev`: adapter bắt input global và phát output qua OS
- `cli`: entrypoint chạy ứng dụng

## Trạng thái hiện tại

Đã hỗ trợ:

- `mouse -> key`
- `key -> mouse`
- `tap`
- `hold`

Ví dụ:
- bấm chuột trái → gửi phím `f`
- bấm `F1` → click chuột trái

> Trên Linux, project hiện ưu tiên **X11**.  
> Nếu bạn đang dùng **Wayland**, chức năng giả lập chuột/phím có thể không hoạt động đúng.

---

## Cấu trúc project

```text
xbuttonmousecontrol/
├─ Cargo.toml
├─ Cargo.lock
├─ config/
│  └─ bindings.toml
├─ crates/
│  ├─ core/
│  ├─ config-toml/
│  ├─ platform-rdev/
│  └─ cli/
├─ .github/workflows/ci.yml
├─ Dockerfile
└─ README.md
````

---

## Yêu cầu môi trường

## 1. Rust

Cài Rust bằng `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Kiểm tra:

```bash
cargo --version
rustc --version
```

## 2. Linux: dependency X11

Trên Ubuntu/Debian:

```bash
sudo apt-get update
sudo apt-get install -y \
  pkg-config \
  libx11-dev \
  libxtst-dev \
  libxi-dev \
  libxdo-dev
```

---

## Chạy project

### Chạy từ source

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

---

## Format cấu hình

File mặc định:

```text
config/bindings.toml
```

Mỗi binding có 5 field:

* `source_type`: loại input nguồn
* `source`: input nguồn
* `target_type`: loại output đích
* `target`: output đích
* `action`: `tap` hoặc `hold`

### Ví dụ đầy đủ

```toml
# mouse -> key
[[bindings]]
source_type = "mouse"
source = "left"
target_type = "key"
target = "f"
action = "tap"

[[bindings]]
source_type = "mouse"
source = "right"
target_type = "key"
target = "h"
action = "tap"

# key -> mouse
[[bindings]]
source_type = "key"
source = "f1"
target_type = "mouse"
target = "left"
action = "tap"

[[bindings]]
source_type = "key"
source = "f2"
target_type = "mouse"
target = "right"
action = "tap"
```

---

## Ý nghĩa các field

### `source_type`

Giá trị hỗ trợ:

* `mouse`
* `key`

### `target_type`

Giá trị hỗ trợ:

* `mouse`
* `key`

### `source` và `target` khi là chuột

Giá trị hỗ trợ:

* `left`
* `right`
* `middle`
* `back`
* `forward`

Alias:

* `x1`
* `x2`
* `mouse4`
* `mouse5`

### `source` và `target` khi là phím

Ví dụ:

* `f1` ... `f12`
* `a` ... `z`
* `0` ... `9`
* `space`
* `enter`
* `tab`
* `esc`
* `ctrl`
* `shift`
* `alt`

### `action`

* `tap`:

    * nếu source được nhấn xuống, app phát 1 lần target
    * release sẽ bị bỏ qua

* `hold`:

    * press source → giữ target
    * release source → nhả target

---

## Ví dụ sử dụng

### 1. Chuột trái thành phím `f`

```toml
[[bindings]]
source_type = "mouse"
source = "left"
target_type = "key"
target = "f"
action = "tap"
```

### 2. Chuột phải thành phím `h`

```toml
[[bindings]]
source_type = "mouse"
source = "right"
target_type = "key"
target = "h"
action = "tap"
```

### 3. F1 click chuột trái

```toml
[[bindings]]
source_type = "key"
source = "f1"
target_type = "mouse"
target = "left"
action = "tap"
```

### 4. Giữ Ctrl bằng nút chuột Forward

```toml
[[bindings]]
source_type = "mouse"
source = "forward"
target_type = "key"
target = "ctrl"
action = "hold"
```

## Troubleshooting

### Nhấn F1 nhưng không click chuột

Kiểm tra lần lượt:

1. `config/bindings.toml` có đúng không

```toml
[[bindings]]
source_type = "key"
source = "f1"
target_type = "mouse"
target = "left"
action = "tap"
```

2. Có đang chạy trên Wayland không

```bash
echo "$XDG_SESSION_TYPE"
```

3. Có build đúng binary mới nhất chưa

```bash
cargo clean
cargo run -p xbuttonmousecontrol-cli -- config/bindings.toml
```

### Linux build lỗi thiếu `xi.pc`

Cài:

```bash
sudo apt-get update
sudo apt-get install -y libxi-dev
```

## Ghi chú

Đây là bản nền tốt để phát triển tiếp theo hướng:

* intercept / consume event thật
* profile switching
* GUI / tray app
* hot reload config
* macro nhiều bước
* guard chống loop giữa `key -> mouse` và `mouse -> key`
