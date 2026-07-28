# 番茄小说下载客户端

基于 Rust + Tauri v2 的跨平台番茄小说下载客户端，支持小说搜索、章节选择下载、书架管理和在线阅读。

![Tauri](https://img.shields.io/badge/Tauri-v2-blue)
![Rust](https://img.shields.io/badge/Rust-stable-orange)
![Vue](https://img.shields.io/badge/Vue-3-brightgreen)
![License](https://img.shields.io/badge/License-MIT-green)
![Release](https://img.shields.io/badge/Release-v0.1.0-success)

## 下载安装

> **Windows 用户**：点击下方链接下载安装包，双击运行即可安装。

[![下载安装包](https://img.shields.io/badge/⬇_下载安装包-v0.1.0-brightgreen?style=for-the-badge&logo=github)](https://github.com/lisiyang2004-pixel/fanqie-novel-client/releases/download/v0.1.0/fanqie-novel-client_0.1.0_x64-setup.exe)

- **安装包下载**：[fanqie-novel-client_0.1.0_x64-setup.exe](https://github.com/lisiyang2004-pixel/fanqie-novel-client/releases/download/v0.1.0/fanqie-novel-client_0.1.0_x64-setup.exe) (3.85 MB)
- **所有版本**：[Releases 页面](https://github.com/lisiyang2004-pixel/fanqie-novel-client/releases)
- **系统要求**：Windows 10/11（需预装 WebView2 运行时）

## 项目简介



本项目是一个使用 Tauri v2 构建的桌面应用，用于从番茄小说网站搜索、浏览和下载小说内容。项目绕过了番茄小说的反爬机制（包括 API 签名认证和 PUA 字符替换），实现了完整的小说下载和管理功能。

### 核心特性

- **多方式搜索**：支持书名/作者关键词搜索（基于必应搜索引擎）、书籍 ID、书籍 URL 三种搜索方式
- **章节选择下载**：可自由选择需要下载的章节，支持全选、反选、范围选择
- **PUA 字符反爬还原**：通过字体解析和上下文推断，建立 288+ 个 PUA 字符映射，还原被加密的汉字
- **双格式导出**：支持 TXT 纯文本和 EPUB 电子书格式（含目录导航）
- **自定义下载位置**：可在设置中指定下载目录到任意盘符
- **书架管理**：添加/移除书籍，跟踪更新
- **下载历史**：记录所有下载任务，支持打开文件所在目录
- **跨平台**：支持 Windows、macOS、Linux

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri v2.11 |
| 后端语言 | Rust (edition 2021) |
| 前端框架 | Vue 3.5 + TypeScript + Vite 6 |
| UI 组件库 | Naive UI 2.40 |
| 状态管理 | Pinia 2.3 |
| 路由 | Vue Router 4.5 |
| 数据库 | SQLite (rusqlite) |
| HTTP 客户端 | reqwest |
| EPUB 生成 | zip crate |
| 搜索引擎 | Bing (书名搜索) |

## 项目结构

```
fanqie-novel-client/
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs               # 程序入口
│   │   ├── lib.rs                # 库入口（插件注册、命令注册）
│   │   ├── commands.rs           # Tauri 命令（前端可调用）
│   │   ├── error.rs              # 统一错误类型
│   │   ├── models.rs             # 数据模型
│   │   ├── api/                  # 番茄小说数据获取
│   │   │   ├── client.rs         # HTTP 客户端
│   │   │   ├── search.rs         # 搜索（必应搜索引擎 + 书籍 ID/URL）
│   │   │   ├── book.rs           # 书籍详情/章节目录（HTML 解析）
│   │   │   └── chapter.rs        # 章节内容获取 + PUA 字符还原
│   │   ├── db/                   # SQLite 数据库
│   │   │   ├── mod.rs            # 数据库连接管理
│   │   │   ├── bookshelf.rs      # 书架操作
│   │   │   ├── history.rs        # 下载历史
│   │   │   └── progress.rs       # 阅读进度
│   │   └── downloader/           # 下载器
│   │       ├── mod.rs            # 下载调度
│   │       ├── txt.rs            # TXT 生成
│   │       └── epub.rs           # EPUB 生成
│   ├── migrations/
│   │   └── 001_init.sql          # 数据库初始化脚本
│   ├── icons/                    # 应用图标
│   ├── Cargo.toml                # Rust 依赖
│   └── tauri.conf.json           # Tauri 配置
├── src/                          # Vue 前端
│   ├── main.ts                   # 应用入口
│   ├── App.vue                   # 根组件（侧边栏布局）
│   ├── router/index.ts           # 路由配置
│   ├── types/index.ts            # TypeScript 类型定义
│   ├── api/index.ts              # Tauri 命令封装
│   ├── stores/                   # Pinia 状态管理
│   │   ├── bookshelf.ts          # 书架状态
│   │   ├── download.ts           # 下载状态
│   │   ├── reader.ts             # 阅读器设置
│   │   └── settings.ts           # 应用设置（下载位置等）
│   ├── views/                    # 页面
│   │   ├── Search.vue            # 搜索页
│   │   ├── BookDetail.vue        # 书籍详情 + 章节选择
│   │   ├── Reader.vue            # 阅读器
│   │   ├── Bookshelf.vue         # 书架
│   │   ├── History.vue           # 下载历史
│   │   └── Settings.vue          # 设置页
│   ├── components/
│   │   └── BookCard.vue          # 书籍卡片组件
│   └── styles/main.css           # 全局样式
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## 核心技术实现

### 1. 绕过 API 签名认证

番茄小说的 API 接口需要 `a_bogus`、`msToken` 等签名参数，直接调用会返回 `invalid client` 错误。本项目改为解析 SSR 页面的 `window.__INITIAL_STATE__` 数据，从 HTML 中提取书籍信息和章节内容。

### 2. PUA 字符反爬还原

番茄小说使用 Unicode 私用区（PUA, U+E000-U+F8FF）字符替换部分汉字进行反爬，纯文本提取时这些字符不可见或显示为方块。

**解决方案**：
- 通过多章节明文-密文对齐提取基础映射（240 个）
- 通过字体文件解析（像素比对法）扩展映射
- 通过上下文推断补充剩余映射
- 当前映射表共 288+ 个 PUA 字符

```rust
// 示例：PUA 字符还原
static PUA_MAP: Lazy<HashMap<u32, char>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(0xE3E9, '在');
    m.insert(0xE3EC, '家');
    // ... 共 288+ 个映射
    m
});
```

### 3. 书名搜索（基于必应）

番茄小说没有公开的书名搜索 API，本项目通过必应搜索引擎间接实现：

- 查询策略 1：`site:fanqienovel.com/page [书名]`
- 查询策略 2：`[书名] 番茄小说 fanqienovel.com/page`
- 从搜索结果中提取书籍 ID

### 4. 章节选择下载

支持灵活的章节选择：
- 全选/反选
- 范围选择（如第 10-50 章）
- 单击选择/取消
- 实时显示已选章节数和总字数

## 开发环境要求

1. **Node.js** >= 18
2. **Rust** >= 1.77（通过 [rustup](https://rustup.rs/) 安装）
3. **系统依赖**：
   - **Windows**: Microsoft Visual Studio C++ Build Tools + WebView2
   - **macOS**: Xcode Command Line Tools
   - **Linux**: `webkit2gtk-4.1`, `libayatana-appindicator3-dev` 等

## 快速开始

### 1. 克隆项目

```bash
git clone https://github.com/你的用户名/fanqie-novel-client.git
cd fanqie-novel-client
```

### 2. 安装依赖

```bash
# 安装前端依赖
npm install

# Rust 依赖会在首次构建时自动下载
```

### 3. 开发模式运行

```bash
npm run tauri:dev
```

### 4. 构建生产版本

```bash
# 构建 NSIS 安装包（Windows）
npm run tauri:build

# 或指定打包格式
npx tauri build --bundles nsis
npx tauri build --bundles msi
```

构建产物位于 `src-tauri/target/release/bundle/` 目录下。

## 使用说明

1. **搜索小说**：在搜索页输入书名、作者、书籍 ID 或书籍 URL
2. **选择章节**：在书籍详情页勾选需要下载的章节
3. **设置下载位置**：在设置页指定下载目录
4. **管理书架**：点击书籍详情页的"加入书架"按钮
5. **查看历史**：在历史页查看所有下载记录

## 阅读器快捷键

| 按键 | 功能 |
|------|------|
| `←` | 上一章 |
| `→` | 下一章 |
| `Esc` | 返回 |

## 数据库

使用 SQLite 存储以下数据：

| 表名 | 用途 |
|------|------|
| `bookshelf` | 书架书籍列表及阅读进度 |
| `download_history` | 下载历史记录 |
| `reading_progress` | 详细阅读进度（章节、滚动位置） |

## 注意事项

1. **API 可用性**：番茄小说的 Web 接口和反爬机制可能随时变更，如遇功能失效需更新代码。
2. **PUA 映射**：字体文件名固定为 `dc027189e0ba4cd`，映射表也固定；如番茄更换字体文件，需重新生成映射表。
3. **限流策略**：批量下载章节时内置请求间隔，避免被服务端限流。
4. **版权声明**：本工具仅供学习交流使用，下载的小说内容版权归原版权方所有，请在 24 小时内删除。

## 许可证

MIT
