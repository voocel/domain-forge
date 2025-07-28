# Domain Forge

**域名生成和可用性检测工具**

一个CLI工具，生成域名并实时检查可用性。

[English](README.md) | 中文文档

## 功能特色

- **域名生成**: 使用 OpenAI、Anthropic、Gemini 或 Ollama 生成域名
- **实时可用性检测**: 使用 RDAP 和 WHOIS 协议检查域名可用性
- **美观的终端界面**: 使用 inquire 库实现的交互式多选界面
- **简单快速**: 最少配置，最高效率
- **多提供商支持**: 支持 OpenAI、Anthropic、Google Gemini 和 Ollama

## 快速开始

### 1. 安装

```bash
git clone https://github.com/voocel/domain-forge.git
cd domain-forge
cargo build --release
```

### 2. 设置API密钥

选择一个支持的提供商：

```bash
# OpenAI (推荐)
export OPENAI_API_KEY="your-openai-api-key"

# Anthropic
export ANTHROPIC_API_KEY="your-anthropic-api-key"

# Gemini
export GEMINI_API_KEY="your-gemini-api-key"

# Ollama (本地，无需API密钥)
# 只需确保Ollama正在运行
```

### 3. 运行

```bash
# 生成随机域名（无需输入）
./target/release/domain-forge

# 为你的想法生成域名
./target/release/domain-forge "AI驱动的生产力应用"
```

## 工作原理

### 无参数运行（随机生成）
```bash
./target/release/domain-forge
```
- 为商业想法生成域名
- 显示美观的交互式选择界面
- 检查选中域名的可用性

### 带描述运行
```bash
./target/release/domain-forge "可持续时尚品牌"
```
- 根据你的描述生成域名
- 交互式多选界面
- 实时可用性检查

## 交互界面

工具提供美观的终端界面：

```
Domain Forge - 域名生成
═══════════════════════════════════════════════════

✅ OpenAI 提供商已配置
→ 为以下内容生成域名: "生产力工具"
⏳ 处理请求中...

生成的域名:
═══════════════════
1. productiv.com (评分: 85%)
   分析: 简短易记的域名

2. taskforge.io (评分: 92%)
   分析: 结合任务管理概念

? 选择要检查可用性的域名:
❯ ◯ 🔄 生成更多选项
  ◯ productiv.com (85%)
  ◯ taskforge.io (92%)
  ◯ ✅ 检查所有域名
```

## 配置

### 环境变量

```bash
# API密钥（选择一个或多个）
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"
export GEMINI_API_KEY="your-key"

# 可选：自定义模型
export OPENAI_MODEL="gpt-4.1-mini"
export ANTHROPIC_MODEL="claude-4-sonnet"
export GEMINI_MODEL="gemini-2.5-flash"
export OLLAMA_MODEL="deepseek-r1"
```

### 支持的提供商

| 提供商 | 模型 | 说明 |
|----------|--------|-------|
| **OpenAI** | gpt-4.1, gpt-4.1-mini, o3, o4-mini | 推荐选择 |
| **Anthropic** | claude-3.7-sonnet, claude-4-sonnet | 备选方案 |
| **Gemini** | gemini-2.5-pro, gemini-2.5-flash | 经济选择 |
| **Ollama** | deepseek-r1, deepseek-v3, qwen3 | 本地部署 |

## 示例

### 创业想法
```bash
./target/release/domain-forge "金融科技移动应用"
./target/release/domain-forge "可持续能源平台"
./target/release/domain-forge "AI驱动的医疗保健"
```

### 创意项目
```bash
./target/release/domain-forge "独立游戏工作室"
./target/release/domain-forge "数字艺术市场"
./target/release/domain-forge "音乐流媒体服务"
```

## 项目结构

```
domain-forge/
├── src/
│   ├── main.rs           # 主程序入口
│   ├── lib.rs            # 库入口
│   ├── types.rs          # 类型定义
│   ├── error.rs          # 错误处理
│   ├── domain/           # 域名检测模块
│   │   ├── mod.rs
│   │   ├── checker.rs
│   │   ├── rdap.rs
│   │   ├── whois.rs
│   │   └── validator.rs
│   └── llm/              # LLM模块
│       ├── mod.rs
│       ├── generator.rs
│       └── providers/    # 每个provider独立文件
│           ├── mod.rs
│           ├── openai.rs
│           ├── anthropic.rs
│           ├── gemini.rs
│           └── ollama.rs
├── Cargo.toml            # 依赖配置
├── README.md             # 项目文档
└── demo.sh               # 演示脚本
```

## 开发

### 从源码构建
```bash
git clone https://github.com/voocel/domain-forge.git
cd domain-forge
cargo build --release
```

### 运行测试
```bash
cargo test
```

### 代码检查
```bash
cargo check
cargo clippy
```

## 要求

- Rust 1.70+
- 至少一个支持的提供商的API密钥
- 用于域名检查的互联网连接

## 贡献

1. Fork 仓库
2. 创建功能分支
3. 进行更改
4. 充分测试
5. 提交拉取请求

## 许可证

Apache 许可证 - 详见 [LICENSE](LICENSE) 文件。

---

**用 Rust 制作**
