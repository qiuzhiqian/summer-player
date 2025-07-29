# Summer Player 构建指南

这个文档详细说明了如何从源码编译 Summer Player。

## 编译依赖

### Linux 系统依赖

#### Debian/Ubuntu
```bash
# 更新包列表
sudo apt update

# 安装基础构建工具
sudo apt install -y build-essential pkg-config

# 安装音频相关依赖
sudo apt install -y libasound2-dev

# 安装 GUI 相关依赖
sudo apt install -y libfontconfig1-dev libxkbcommon-dev

# 可选：安装额外的开发工具
sudo apt install -y git curl
```

#### Fedora/RHEL/CentOS
```bash
# 安装基础构建工具
sudo dnf groupinstall "Development Tools"
sudo dnf install pkg-config

# 安装音频相关依赖
sudo dnf install alsa-lib-devel

# 安装 GUI 相关依赖
sudo dnf install fontconfig-devel libxkbcommon-devel
```

#### Arch Linux
```bash
# 安装基础构建工具
sudo pacman -S base-devel

# 安装依赖
sudo pacman -S alsa-lib fontconfig libxkbcommon
```

#### openSUSE
```bash
# 安装基础构建工具
sudo zypper install -t pattern devel_basis

# 安装依赖
sudo zypper install alsa-devel fontconfig-devel libxkbcommon-devel
```

### Rust 工具链

```bash
# 安装 Rust (推荐使用 rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重新加载环境变量
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

## 构建过程

### 1. 克隆仓库

```bash
git clone <repository-url>
cd summer-player
```

### 2. 常规编译

```bash
# Debug 构建
cargo build

# Release 构建（推荐）
cargo build --release

# 运行
cargo run --release
```

### 3. 构建 deb 包

```bash
# 安装 cargo-deb
cargo install cargo-deb

# 构建 deb 包
cargo deb

# 安装构建的包
sudo dpkg -i target/debian/summer-player_0.3.0-1_amd64.deb
```

## 故障排除

### 常见问题

#### 1. ALSA 相关错误
```
error: failed to run custom build command for `alsa-sys`
```

**解决方案**：
```bash
sudo apt install libasound2-dev
```

#### 2. 字体配置错误
```
error: failed to run custom build command for `fontconfig-sys`
```

**解决方案**：
```bash
sudo apt install libfontconfig1-dev
```

#### 3. 键盘处理错误
```
error: failed to run custom build command for `xkbcommon-sys`
```

**解决方案**：
```bash
sudo apt install libxkbcommon-dev
```

#### 4. pkg-config 错误
```
error: could not find system library 'pkg-config'
```

**解决方案**：
```bash
sudo apt install pkg-config
```

### 验证构建

构建完成后，可以验证二进制文件：

```bash
# 检查二进制文件信息
file target/release/summer_player

# 检查依赖库
ldd target/release/summer_player

# 运行测试
./target/release/summer_player --help
```

## 交叉编译

### 为不同架构编译

```bash
# 添加目标架构
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu

# 交叉编译
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

## 优化构建

### 减小二进制大小

在 `Cargo.toml` 中添加：

```toml
[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### 并行编译

```bash
# 使用所有 CPU 核心
cargo build --release -j $(nproc)
```

## 打包分发

### 创建 AppImage（可选）

```bash
# 安装 cargo-appimage
cargo install cargo-appimage

# 构建 AppImage
cargo appimage
```

### 创建 Flatpak（可选）

需要创建 Flatpak 清单文件，参考官方文档。

## 开发环境

### IDE 设置

推荐使用以下开发环境：
- VS Code + rust-analyzer
- IntelliJ IDEA + Rust 插件
- Vim/Neovim + rust.vim

### 代码检查

```bash
# 代码格式化
cargo fmt

# 代码检查
cargo clippy

# 运行测试
cargo test
```

## 性能优化

### PGO（配置文件引导优化）

```bash
# 1. 构建带插桩的版本
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
    cargo build --release

# 2. 运行程序生成配置文件
./target/release/summer_player

# 3. 使用配置文件重新构建
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data -Cllvm-args=-pgo-warn-missing-function" \
    cargo build --release
```

---

如有问题，请提交 Issue 或查看项目文档。 