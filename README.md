# xbuttonmousecontrol

Ứng dụng remap **phím ↔ chuột** trên **Windows** và **Linux/X11**.

Hiện tại:
- **Windows**: dùng backend `platform-win-hook`
- **Linux**: dùng backend `platform-x11-grab`
- **Wayland**: chưa phải môi trường ưu tiên, nên có thể không hoạt động đúng trên Linux nếu bạn đang đăng nhập bằng Wayland. :contentReference[oaicite:1]{index=1}

---

# 1. Yêu cầu môi trường

## 1.1. Rust

Cần cài Rust để chạy project từ source.

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
````

### Windows

Cài Rust bằng `rustup-init.exe` từ trang Rust chính thức, sau đó mở terminal mới.

### Kiểm tra

```bash
cargo --version
rustc --version
```

---

## 1.2. Linux: thư viện hệ thống cần thiết

Trên Linux, project hiện build theo hướng **X11**.
Cài các package sau trên Ubuntu/Debian:

```bash
sudo apt-get update
sudo apt-get install -y \
  pkg-config \
  libx11-dev \
  libxtst-dev \
  libxi-dev \
  libxdo-dev
```

Các dependency này cũng chính là những gói đang được dùng trong CI và Dockerfile của project.

> Lưu ý:
>
> * Nếu bạn đang dùng **Wayland**, chức năng hook / giả lập chuột phím có thể không hoạt động đúng.
> * Nên test bằng phiên **X11** trên Linux.

---

# 2. Cài source code

Clone project:

```bash
git clone <YOUR_REPOSITORY_URL>
cd xbuttonmousecontrol
```

---

# 3. Cách chạy khi không dùng file `.exe` hay file build sẵn

## 3.1. Chạy trực tiếp từ source

```bash
cargo run -p xbuttonmousecontrol-cli -- config/bindings.toml
```

Lệnh này dùng đúng binary của crate `xbuttonmousecontrol-cli` và truyền file config vào làm tham số. Đây cũng là cách chạy được README hiện tại dùng để test.

---

## 3.2. Build rồi chạy bằng binary tự build

### Linux

```bash
cargo build --release -p xbuttonmousecontrol-cli
./target/release/xbuttonmousecontrol-cli config/bindings.toml
```

### Windows

```powershell
cargo build --release -p xbuttonmousecontrol-cli
.\target\release\xbuttonmousecontrol-cli.exe config\bindings.toml
```

Các lệnh này đang khớp với README và workflow build hiện tại của project.

---

# 4. File cấu hình nằm ở đâu

File cấu hình mặc định là:

```text
config/bindings.toml
```

Ngoài ra, chương trình còn tự tìm file config theo thứ tự sau:

1. đường dẫn truyền qua tham số dòng lệnh
2. `./config/bindings.toml`
3. `./bindings.toml`
4. `<exe_dir>/config/bindings.toml`
5. `<exe_dir>/bindings.toml`

Vì vậy, cách an toàn nhất là luôn chạy kèm đường dẫn rõ ràng:

### Linux

```bash
cargo run -p xbuttonmousecontrol-cli -- ./config/bindings.toml
```

### Windows

```powershell
.\target\release\xbuttonmousecontrol-cli.exe .\config\bindings.toml
```

---

# 5. Cấu trúc file cấu hình

File cấu hình dùng định dạng **TOML**.
Danh sách binding được đặt trong mảng `[[bindings]]`.

Ví dụ:

```toml
[[bindings]]
source_type = "key"
source = "f1"
target_type = "mouse"
target = "left"
action = "tap"
mode = "replace"
```

Mỗi binding gồm các trường:

* `source_type`
* `source`
* `target_type`
* `target`
* `action`
* `mode` (không bắt buộc, có giá trị mặc định)

---

# 6. Giải thích các trường cấu hình

## 6.1. `source_type`

Loại input nguồn.

Giá trị hỗ trợ:

* `"key"`
* `"mouse"`

Ví dụ:

```toml
source_type = "key"
```

hoặc

```toml
source_type = "mouse"
```

---

## 6.2. `source`

Tên của input nguồn.

Nếu `source_type = "mouse"` thì các giá trị hỗ trợ là:

* `left`
* `right`
* `middle`
* `back`
* `forward`

Alias tương đương:

* `x1` = `back`
* `mouse4` = `back`
* `x2` = `forward`
* `mouse5` = `forward`

Ví dụ:

```toml
source = "left"
```

Nếu `source_type = "key"` thì parser hiện nhận chuỗi phím dưới dạng text thường hóa về lowercase. Trong README cũ, các ví dụ đã dùng các tên như:

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

Ví dụ:

```toml
source = "f1"
```

---

## 6.3. `target_type`

Loại output đích.

Giá trị hỗ trợ:

* `"key"`
* `"mouse"`

Ví dụ:

```toml
target_type = "mouse"
```

---

## 6.4. `target`

Tên output đích.

Nếu `target_type = "mouse"` thì giá trị hỗ trợ giống phần `source` chuột:

* `left`
* `right`
* `middle`
* `back`
* `forward`

Alias:

* `x1`
* `mouse4`
* `x2`
* `mouse5`

Ví dụ:

```toml
target = "left"
```

Nếu `target_type = "key"` thì dùng tên phím dạng text, ví dụ:

```toml
target = "f"
```

hoặc:

```toml
target = "ctrl"
```

---

## 6.5. `action`

Cách phát output khi input xảy ra.

Giá trị hỗ trợ:

* `"tap"`
* `"hold"`

### `tap`

Khi nhấn input nguồn, app phát output **một lần**.
Sự kiện release của nguồn sẽ bị bỏ qua.

Ví dụ:

```toml
action = "tap"
```

### `hold`

Khi nhấn input nguồn, app giữ output.
Khi nhả input nguồn, app nhả output.

Ví dụ:

```toml
action = "hold"
```

---

## 6.6. `mode`

Cách xử lý input gốc khi binding match.

Trường này là **không bắt buộc**.
Nếu không ghi, giá trị mặc định là:

```toml
mode = "additive"
```

Các giá trị parser hiện hỗ trợ:

* `additive`
* `passthrough`
* `replace`
* `suppress`

Ý nghĩa:

### `additive`

Hoặc `passthrough`

* giữ nguyên input gốc
* đồng thời phát thêm target

Ví dụ:

* bấm `F1`
* app vẫn cho `F1` đi qua
* đồng thời phát thêm click chuột trái

### `replace`

Hoặc `suppress`

* chặn input gốc
* thay bằng target mới

Ví dụ:

* bấm `F1`
* app chặn `F1`
* thay bằng click chuột trái

Đây là chế độ dùng khi bạn muốn “nút cũ mất chức năng, chỉ còn chức năng mới”.

---

# 7. Ví dụ cấu hình

## 7.1. F1 thành click chuột trái, chặn luôn F1 gốc

```toml
[[bindings]]
source_type = "key"
source = "f1"
target_type = "mouse"
target = "left"
action = "tap"
mode = "replace"
```

---

## 7.2. F2 thành click chuột phải, nhưng vẫn giữ F2 gốc

```toml
[[bindings]]
source_type = "key"
source = "f2"
target_type = "mouse"
target = "right"
action = "tap"
mode = "additive"
```

---

## 7.3. Chuột trái thành phím `f`

```toml
[[bindings]]
source_type = "mouse"
source = "left"
target_type = "key"
target = "f"
action = "tap"
mode = "replace"
```

---

## 7.4. Nút chuột forward giữ `ctrl`

```toml
[[bindings]]
source_type = "mouse"
source = "forward"
target_type = "key"
target = "ctrl"
action = "hold"
mode = "replace"
```

---

# 8. File cấu hình mẫu đầy đủ

```toml
[[bindings]]
source_type = "key"
source = "f1"
target_type = "mouse"
target = "left"
action = "tap"
mode = "replace"

[[bindings]]
source_type = "key"
source = "f2"
target_type = "mouse"
target = "right"
action = "tap"
mode = "replace"

[[bindings]]
source_type = "mouse"
source = "back"
target_type = "key"
target = "ctrl"
action = "hold"
mode = "replace"

[[bindings]]
source_type = "mouse"
source = "forward"
target_type = "key"
target = "enter"
action = "tap"
mode = "additive"
```

---

# 9. Troubleshooting nhanh

## 9.1. Linux không hoạt động

Kiểm tra session hiện tại:

```bash
echo "$XDG_SESSION_TYPE"
```

Nếu ra `wayland`, app có thể không hoạt động đúng vì backend Linux hiện đang đi theo hướng X11.

---

## 9.2. App không nhận đúng file config

Hãy truyền đường dẫn file config rõ ràng khi chạy:

### Linux

```bash
cargo run -p xbuttonmousecontrol-cli -- ./config/bindings.toml
```

### Windows

```powershell
.\target\release\xbuttonmousecontrol-cli.exe .\config\bindings.toml
```

Vì app hiện có cơ chế tự dò nhiều vị trí file cấu hình.

---

## 9.3. Linux build lỗi thiếu package

Cài lại:

```bash
sudo apt-get update
sudo apt-get install -y \
  pkg-config \
  libx11-dev \
  libxtst-dev \
  libxi-dev \
  libxdo-dev
```