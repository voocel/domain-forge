# Domain Forge

**域名生成和可用性检测工具**

一个CLI工具，生成域名并实时检查可用性。

[English](README.md) | 中文文档

## 功能特色

- **AI域名生成**: 使用 OpenAI、Anthropic、Gemini 或 Ollama 生成域名
- **实时可用性检测**: 使用 RDAP 和 WHOIS 协议检查域名可用性
- **域名捡漏**: 扫描可用的短域名（4字母或5字母有意义单词）
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

# 扫描5字母有意义词域名（推荐！）
./target/release/domain-forge snipe -w --tld com
```

## 域名捡漏

使用 `snipe` 命令扫描可用的短域名：

### 扫描模式

| 模式 | 参数 | 域名数量 | 说明 |
|------|------|----------|------|
| 全量扫描 | (无) | ~456k | 所有4字母组合 (aaaa-zzzz) |
| N字母扫描 | `-l N` | 可变 | 所有N字母组合 (2-10) |
| 可发音 | `-p` | ~137k | 4字母可发音模式 (CVCV等) |
| **词库** | `-w` | ~10k | 5字母有意义单词（推荐！） |
| **可读** | `-R` | ~27k | 5字母可读/品牌化名称 (CVCVC模式) |
| 6字母 | `--six` | ~351k | 6字母可发音模式 |

### 使用方法

```bash
# 5字母有意义单词（推荐！）
./target/release/domain-forge snipe -w --tld com

# 3字母 .ai 域名
./target/release/domain-forge snipe -l 3 --tld ai

# 4字母可发音模式
./target/release/domain-forge snipe -p --tld com

# 4字母全量扫描
./target/release/domain-forge snipe --tld com

# 6字母可发音模式
./target/release/domain-forge snipe --six --tld com

# 5字母可读/品牌化名称 (CVCVC模式, ~27k)
./target/release/domain-forge snipe -R --tld com

# 扫描多个TLD
./target/release/domain-forge snipe -w --tld com,io,ai

# 调整并发和速率限制（针对限流严格的服务器如 .ai）
./target/release/domain-forge snipe -l 3 --tld ai -c 5 --rate 1000

# 恢复中断的扫描
./target/release/domain-forge snipe -w -r
```

### 按 TLD 推荐的最优命令

```bash
# .com / .net / .org（服务器快，可用高并发）
./target/release/domain-forge snipe -l 3 --tld com -c 20 --rate 200
./target/release/domain-forge snipe -p --tld com -c 20 --rate 200

# .ai（限流严格，使用保守设置）
./target/release/domain-forge snipe -l 3 --tld ai -c 5 --rate 1000
./target/release/domain-forge snipe -l 4 --tld ai -c 3 --rate 2000

# .io / .co / .me（中等限流）
./target/release/domain-forge snipe -l 3 --tld io -c 10 --rate 500

# 多 TLD 扫描（使用保守设置）
./target/release/domain-forge snipe -w --tld com,io,ai -c 10 --rate 500
```

### 参数说明

| 参数 | 说明 |
|------|------|
| `-l, --length <N>` | 域名长度 (2-10，默认: 4) |
| `-w, --words` | 扫描5字母有意义单词（推荐） |
| `-R, --readable` | 扫描5字母可读/品牌化名称 (~27k) |
| `-p, --pronounceable` | 扫描4字母可发音模式 |
| `--six` | 扫描6字母可发音模式 |
| `-t, --tld <TLD>` | 要扫描的TLD（逗号分隔，默认: com） |
| `-a, --alphanumeric` | 包含数字 (a-z, 0-9) |
| `-c, --concurrency <N>` | 并发数（默认: 20） |
| `--rate <MS>` | 批次间延迟毫秒数（默认: 500） |
| `-r, --resume` | 恢复上次扫描 |
| `-e, --expiring <DAYS>` | 即将过期天数阈值（默认: 7） |

### 重新检查结果

```bash
# 重新检查并更新已保存的结果
./target/release/domain-forge snipe recheck output/snipe_results_*.json
```

### 词库内容

5字母词库包含约10,000个高价值域名：

- **常用词汇**: cloud, pixel, forge, spark, alpha...
- **技术词汇**: bytes, nodes, cache, async, react...
- **品牌化词汇**: zippy, happy, bingo, turbo, promo...
- **品牌风格**: ifish, ebook, xcode, uplay, myapp...

结果保存在 `output/` 目录中。

## AI域名生成

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

OpenAI 提供商已配置
→ 为以下内容生成域名: "生产力工具"
处理请求中...

生成的域名:
═══════════════════
1. productiv.com (评分: 85%)
   分析: 简短易记的域名

2. taskforge.io (评分: 92%)
   分析: 结合任务管理概念

? 选择要检查可用性的域名:
❯ ◯ 生成更多选项
  ◯ productiv.com (85%)
  ◯ taskforge.io (92%)
  ◯ 检查所有域名
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
