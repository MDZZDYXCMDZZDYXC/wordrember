# WordRember - 背单词应用

基于 Tauri 技术栈的跨平台背单词应用，支持桌面端和移动端（Android）。

## 项目简介

WordRember 是一款简洁高效的背单词工具，采用现代化的技术栈开发：
- **前端**: 原生 HTML + CSS + JavaScript
- **后端**: Rust + Tauri v2
- **数据库**: SQLite
- **跨平台**: Windows, macOS, Linux, Android

## 功能特性

### 1. 单词学习系统
- **三种视图模式**：
  - 全部单词：查看所有已添加的单词
  - 待掌握单词：显示需要继续学习的单词
  - 已掌握单词：显示已达到记忆阈值的单词
- **记忆状态标记**：支持标记"已掌握"或"忘记"
- **记忆阈值机制**：达到设定次数后自动移出学习列表
- **记忆程度可视化**：通过左侧色条显示记忆等级（红→橙→黄→绿）

### 2. 单词管理功能
- **添加单词**：手动输入单词和意思
- **导入单词**：支持 CSV、JSON、TXT 格式文件导入
- **删除单词**：支持单个删除，自动清理关联数据
- **重置进度**：一键重置所有单词的学习进度

### 3. 界面设计
- **赛博朋克风格**：深色主题 + 霓虹绿配色
- **毛玻璃效果**：半透明卡片和导航栏
- **响应式布局**：自适应不同屏幕尺寸
- **安全区域适配**：适配手机状态栏和虚拟导航栏

### 4. 底部导航栏
- 全部单词
- 待掌握单词
- 已掌握单词
- 单词管理
- 系统设置

## 文件格式说明

### CSV 格式
```
word,meaning
apple,苹果
book,书
```

### JSON 格式
```json
[
  {"word": "apple", "meaning": "苹果"},
  {"word": "book", "meaning": "书"}
]
```

### TXT 格式
```
word	meaning
apple	苹果
book	书
```
（使用 Tab 分隔）

## 项目结构

```
wordrember/
├── src/                    # 前端文件
│   ├── index.html         # 主页面
│   ├── styles.css         # 样式表
│   ├── app.js             # 主逻辑
│   └── back.jpg           # 背景图片
├── src-tauri/             # Rust 后端
│   ├── src/
│   │   ├── lib.rs         # 核心逻辑
│   │   └── main.rs        # Tauri 入口
│   ├── Cargo.toml         # Rust 依赖
│   ├── tauri.conf.json    # Tauri 配置
│   ├── build.rs           # 构建脚本
│   ├── capabilities/      # 权限配置
│   └── icons/            # 应用图标
├── .gitignore
└── README.md
```

## 构建和运行

### 开发模式（桌面）
```bash
cd src-tauri
cargo tauri dev
```

### 构建发布版本（桌面）
```bash
cd src-tauri
cargo tauri build
```

### 构建 Android APK
```bash
cd src-tauri
cargo tauri android init
cargo tauri android build
```

## 技术栈

- **前端**: HTML5, CSS3, JavaScript (ES6+)
- **后端**: Rust, Tauri v2
- **数据库**: SQLite (rusqlite)
- **构建工具**: Cargo
- **移动端**: Android (通过 Tauri)

## 快捷键

- `←` / `→`: 前后翻页
- `ESC`: 关闭弹窗

## 配置选项

在设置中可以调整：
- 记忆阈值（默认5次）
- 每页显示单词数量

## 开发说明

### 数据库结构
- `words` 表：存储单词信息
- `learned_words` 表：存储学习记录
- `settings` 表：存储应用设置

### 外键约束
- `learned_words.word_id` 外键引用 `words.id`
- 删除单词时自动清理关联的学习记录

## 许可证

MIT License