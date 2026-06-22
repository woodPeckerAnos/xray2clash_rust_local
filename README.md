# **[xray2clash_rust_local](https://github.com/woodPeckerAnos/xray2clash_rust_local)**



将 xray 安装脚本生成的 **VLESS + REALITY** 分享链接（`vless://`）转换为 [Clash Verge Dev](https://github.com/clash-verge-rev/clash-verge-rev) 可直接导入的 Mihomo YAML 配置。

本地离线运行，无需第三方在线转换。

## 前置要求

- [Rust](https://www.rust-lang.org/tools/install) 1.70+（`rustup` 或 `brew install rust`）

## 安装

```bash
cd ~/Projects/vless-clash-dev
cargo build --release
```

编译产物：`target/release/vless-clash-dev`

全局安装（可选）：

```bash
cargo install --path .
```

## 用法

### 单条链接 → YAML 文件

```bash
vless-clash-dev \
  --input "vless://UUID@server:443?type=tcp&security=reality&sni=www.example.com&fp=chrome&pbk=PUBLIC_KEY&sid=SHORT_ID&flow=xtls-rprx-vision#NodeName" \
  --output ./outputs/my-node.yaml \
  --mode rule
```

### 批量转换（每行一个链接）

```bash
vless-clash-dev \
  --input-file ./links.txt \
  --output-dir ./outputs \
  --mode both
```

### 白名单模式（仅常用站点走代理）

默认加载全部内置 preset：

```bash
vless-clash-dev --input "vless://..." --output ./outputs/my-node-whitelist.yaml --mode whitelist
```

按需组合 preset：

```bash
vless-clash-dev --input "vless://..." --mode whitelist \
  --preset github --preset android --preset ai \
  --output ./outputs/dev-whitelist.yaml
```

查看可用 preset：

```bash
vless-clash-dev --list-presets
```

### 重命名节点与输出文件

```bash
# 重命名 Clash 中的节点名
vless-clash-dev --input "vless://..." --rename "我的节点" --mode rule --stdout

# 重命名输出文件名（节点名仍取自链接 fragment）
vless-clash-dev --input "vless://..." --mode whitelist \
  --output-name "home-whitelist" \
  --output-dir ./outputs
# → ./outputs/home-whitelist.yaml

# 同时指定节点名和文件名
vless-clash-dev --input "vless://..." --name "US-Node" \
  --output-name "us-proxy" --mode both --output-dir ./outputs
# → ./outputs/us-proxy-global.yaml
# → ./outputs/us-proxy-rule.yaml
# → ./outputs/us-proxy-whitelist.yaml
```

### 输出到 stdout

```bash
vless-clash-dev --input "vless://..." --stdout --mode rule
```

## CLI 参数


| 参数                                  | 说明                                |
| ----------------------------------- | --------------------------------- |
| `--input`                           | 单条 `vless://` 链接                  |
| `--input-file`                      | 文本文件，每行一条链接（`#` 开头为注释）            |
| `--output`                          | 输出 YAML 文件路径（仅单条输入）               |
| `--output-dir`                      | 输出目录（默认 `./outputs`）              |
| `--stdout`                          | 写入标准输出                            |
| `--mode global|rule|whitelist|both` | 生成全局、分流、白名单或全部三种配置（默认 `rule`）     |
| `--preset <name>`                   | 白名单 preset 名称，可重复指定               |
| `--preset-all`                      | 使用全部内置 preset                     |
| `--custom-preset <file>`            | 额外加载自定义 preset YAML，可重复指定         |
| `--list-presets`                    | 列出内置与用户 preset                    |
| `--name`, `--rename`                | 重命名 Clash 中的代理节点                  |
| `--output-name`                     | 重命名输出文件名（不含扩展名，配合 `--output-dir`） |


## 分享链接格式

```
vless://UUID@server:443?type=tcp&security=reality&sni=www.example.com&fp=chrome&pbk=PUBLIC_KEY&sid=SHORT_ID&flow=xtls-rprx-vision#NodeName
```


| 参数            | 说明                          |
| ------------- | --------------------------- |
| `UUID`        | 用户 ID                       |
| `server:port` | 服务器地址与端口                    |
| `security`    | 必须为 `reality`               |
| `sni`         | REALITY 伪装域名                |
| `pbk`         | 服务端公钥                       |
| `fp`          | 浏览器指纹（默认 `chrome`）          |
| `sid`         | Short ID（可选）                |
| `flow`        | 流控，如 `xtls-rprx-vision`（可选） |
| `#NodeName`   | 节点名称（URL 编码）                |


## Clash Verge Dev 导入步骤

1. 打开 **Clash Verge Dev** → **Profiles（配置）**
2. 点击 **Import（导入）** → 选择生成的 `.yaml` 文件
3. 确认内核为 **Mihomo / Meta**（Verge Dev 默认）
4. 启用该配置，在 **PROXY** 组中选择节点

## 输出说明

### rule 模式（默认）

- 局域网/私有 IP → `DIRECT`
- 中国大陆域名/IP → `DIRECT`
- 其余流量 → `PROXY`

### global 模式

- 所有流量走 `PROXY`（适合快速测试单节点）

### whitelist 模式

- 局域网/私有 IP → `DIRECT`
- 所选 preset 中的域名（`DOMAIN-SUFFIX` 规则，无 `GEOSITE`）→ `PROXY`
- 其余流量 → `DIRECT`

内置 preset 按功能拆分在 `[presets/](presets/)` 目录：


| preset         | 说明                   |
| -------------- | -------------------- |
| `google`       | Google、Gmail、YouTube |
| `github`       | GitHub 及 raw 内容      |
| `android`      | Android 开发与构建        |
| `ai`           | OpenAI、Claude 等      |
| `social`       | 社交媒体                 |
| `devtools`     | 开发工具与包仓库             |
| `productivity` | 协作与设计工具              |
| `media`        | 新闻与流媒体               |


未指定 `--preset` 时，默认使用全部内置 preset。自定义 preset 可放到 `~/.config/vless-clash-dev/presets/`，或通过 `--custom-preset` 指定文件。格式见 `[presets/README.md](presets/README.md)`。

## 开发与测试

```bash
cargo test
cargo build --release
```

## 限制

- 仅支持 `vless://` + `security=reality`
- 不支持 vmess、trojan、shadowsocks 等协议
- 不解析服务端 `config.json`
- 不自动写入 Clash Verge Dev 配置目录

## License

MIT