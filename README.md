# punct-converter

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

将文本文件中的中文/全角标点符号批量替换为英文/半角标点符号的轻量命令行工具与 Rust 库.

支持单文件, 批量文件, 递归文件夹与管道输入, 自动探测 UTF-8 与 GB18030(兼容 GBK/GB2312)编码并原样写回, 内置二进制防误伤机制, 跨 Windows / Linux / macOS 通用.

---

## 特性

- **编码智能识别**: 自动探测 UTF-8(含/不含 BOM)与 GB18030 编码, 转换后保持原编码写回(亦支持统一转为 UTF-8).
- **二进制安全防护**: 无法合规解码的数据直接判定为疑似二进制文件并自动跳过, 绝不破坏非文本文件.
- **智能排版间距**: 断句标点(逗号, 句号, 冒号, 分号等)转换后紧邻正文字符时自动补一个半角空格; 对数字之间(如小数 `3.14`, 时间 `12:30`, 千分位 `1,000`)智能免除空格.
- **多字符智能合并**: 中文双省略号 `……` 自动合并替换为 `...`(非重复的 6 个点), 双破折号 `——` 自动合并替换为 `--`.
- **Git 友好**: 目录递归遍历默认遵循 `.gitignore` 规则, 自动跳过隐藏目录与编译产物(`node_modules/`, `target/` 等).
- **灵活输入输出**: 支持命令行参数指定路径, 也支持标准输入(管道模式: `echo "你好，世界！" | punct-converter`).
- **安全预览与备份**: 提供 `--dry-run` 预览修改, `--backup` 生成 `.bak` 备份, 防止误操作.

---

## 安装

### 方式一: 下载预编译二进制(推荐)

前往仓库的 Releases 页面, 下载对应操作系统的压缩包, 解压后将可执行文件放置在系统 `PATH` 路径下即可.

### 方式二: 使用 Cargo 安装

如果本地已配置 Rust 工具链:

```bash
# 从 crates.io 安装 (发布后可用)
cargo install punct-converter

# 或从源码仓库直接安装
cargo install --git https://github.com/5angXR/punct-converter.git
```

### 方式三: 从源码构建

```bash
git clone https://github.com/5angXR/punct-converter.git
cd punct-converter
cargo build --release
```

编译产物位于 `target/release/punct-converter`(Windows 下为 `punct-converter.exe`).

---

## 命令行用法

```text
punct-converter [OPTIONS] [PATHS]...
```

### 常用命令示例

```bash
# 1. 转换单个文件
punct-converter file.md

# 2. 管道模式: 从标准输入读取并输出到标准输出
echo "你好，世界！圆周率3。14，时间12：30……" | punct-converter

# 3. 递归处理整个目录, 先预览将要修改的文件与次数 (不实际写入)
punct-converter -r --dry-run ./docs

# 4. 递归处理目录并在覆盖前备份原文件 (生成 <文件名>.bak)
punct-converter -r --backup ./docs

# 5. 指定只处理特定后缀的文件
punct-converter -r -e md,txt ./docs

# 6. 处理代码/配置文件时, 保留中文引号不转换 (避免破坏字符串字面量)
punct-converter -r --no-quotes ./src

# 7. 不自动补空格, 严格做一对一字符替换
punct-converter --no-space file.txt

# 8. 强制统一以 UTF-8 编码写回文件
punct-converter -r --to-utf8 ./docs
```

### 参数说明

| 参数 / 选项 | 描述 |
|---|---|
| `PATHS` | 一个或多个文件或文件夹路径. 传入 `-` 或通过管道输入时自动走标准输入 |
| `-r, --recursive` | 递归处理子文件夹(默认只扫描文件夹的一级子目录) |
| `-e, --ext <EXT>` | 只处理指定扩展名的文件(逗号分隔, 不区分大小写, 默认涵盖常见文档与文本扩展名) |
| `--dry-run` | 预览模式: 仅打印会被修改的文件和替换次数, 不实际改动文件 |
| `--backup` | 覆盖前把原文件备份为 `<文件名>.bak` |
| `--to-utf8` | 无论原文件是什么编码, 统一转为 UTF-8 编码输出 |
| `--no-space` | 关闭断句标点后自动补空格的行为, 严格一对一符号替换 |
| `--no-quotes` | 跳过双引号(`“”`)与单引号(`‘’`)的转换 |
| `--no-ignore` | 扫描目录时不遵循 `.gitignore` 规则, 亦不跳过隐藏文件 |
| `-h, --help` | 打印帮助信息 |
| `-V, --version` | 打印版本信息 |

---

## 标点映射表

| 中文标点 | 英文对应 | 说明 |
|---|---|---|
| `，` | `,` | 逗号, 紧邻正文时默认补空格 |
| `。` | `.` | 句号, 数字间(如 `3.14`)不补空格 |
| `！` | `!` | 感叹号 |
| `？` | `?` | 问号 |
| `：` | `:` | 冒号, 数字间(如 `12:30`)不补空格 |
| `；` | `;` | 分号 |
| `、` | `,` | 顿号转为英文逗号 |
| `……` / `…` | `...` | 省略号, 双字符智能合并 |
| `——` / `—` | `--` | 破折号, 双字符智能合并 |
| `“”` | `"` | 双引号(可通过 `--no-quotes` 跳过) |
| `‘’` | `'` | 单引号(可通过 `--no-quotes` 跳过) |
| `（）` | `()` | 圆括号 |
| `【】` `〔〕` | `[]` | 方括号 |
| `《》` `〈〉` | `<>` | 书名号/尖括号 |
| `～％＃＆＊＋－＝＠｜＼／＿｀＾｛｝` | `` ~%#&*+-=@\|\/_`^{} `` | 全角符号对应半角字符 |

---

## 作为 Rust 库使用

在你的 `Cargo.toml` 中添加依赖:

```toml
[dependencies]
punct-converter = "0.1"
```

代码调用示例:

```rust
use punct_converter::{convert_text, convert_text_with_options, ConvertOptions};

fn main() {
    // 使用默认选项转换
    let result = convert_text("你好，世界！圆周率3。14，时间12：30……");
    println!("{}", result.text);
    // 输出: "你好, 世界! 圆周率3.14, 时间12:30..."

    // 自定义选项
    let options = ConvertOptions {
        space_after_terminal_punct: false, // 不补空格
        convert_quotes: false,             // 保留中文引号
    };
    let custom = convert_text_with_options("我说：“你好”，他说：‘世界’", &options);
    println!("{}", custom.text);
}
```

---

## 开源协议

本项目采用 [MIT License](LICENSE) 授权许可.
