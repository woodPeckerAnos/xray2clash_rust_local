# Whitelist Presets

按功能划分的域名白名单。转换时使用 `--preset <name>` 按需组合，或使用 `--preset-all` 加载全部内置 preset。

## 内置 preset

| 名称 | 说明 |
|------|------|
| `google` | Google、Gmail、YouTube |
| `github` | GitHub 及 raw 内容 |
| `android` | Android 开发与构建 |
| `ai` | OpenAI、Claude 等 AI 服务 |
| `social` | 社交媒体与即时通讯 |
| `devtools` | 开发工具与包仓库 |
| `productivity` | 协作与设计工具 |
| `media` | 新闻与流媒体 |

## 自定义 preset

在 `~/.config/vless-clash-dev/presets/` 下新建 YAML 文件，或通过 `--custom-preset` 指定路径：

```yaml
name: my-sites
description: Personal whitelist
domains:
  - example.com
  - cursor.com
```

使用方式：

```bash
# 按名称加载自定义 preset（需在配置目录中）
vless-clash-dev --input "vless://..." --mode whitelist --preset my-sites

# 直接指定文件路径
vless-clash-dev --input "vless://..." --mode whitelist --custom-preset ./my-sites.yaml
```

规则说明：

- 仅使用 `DOMAIN-SUFFIX` 规则，不含 `GEOSITE`
- 不包含 Apple 相关域名
- 多个 preset 的域名会自动去重
