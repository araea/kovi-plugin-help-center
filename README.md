kovi-plugin-help-center
=======================

[<img alt="github" src="https://img.shields.io/badge/github-araea/kovi__plugin__help__center-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/araea/kovi-plugin-help-center)
[<img alt="crates.io" src="https://img.shields.io/crates/v/kovi-plugin-help-center.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/kovi-plugin-help-center)

Kovi 的现代化帮助菜单插件。抛弃枯燥的纯文本列表，使用 HTML/CSS 渲染精美的玻璃拟态 UI 图片。

## 特性

- 🎨 **精美 UI** - 玻璃拟态设计风格，支持自定义主题配色
- 📁 **分类管理** - 清晰的插件分类展示，层次分明
- 🔍 **指令搜索** - 支持按插件名或指令关键词模糊搜索
- ⚡ **高性能** - 智能图片缓存机制，避免重复渲染
- 🔄 **热重载** - 修改配置后立即生效，无需重启

## 前置

1. 创建 Kovi 项目
2. 执行 `cargo kovi add help-center`
3. 在 `src/main.rs` 中添加 `kovi_plugin_help_center`

## 快速开始

1. 发送 `帮助` 或 `菜单` 查看主菜单
2. 发送 `帮助 关键词` 搜索特定指令（如 `帮助 签到`）
3. 发送 `分类` 查看所有分类列表

## 指令列表

| 指令 | 别名 | 功能 |
|------|------|------|
| `帮助` | `菜单`, `help` | 查看完整帮助菜单图片 |
| `帮助 <关键词>` | `搜索`, `search` | 搜索相关插件或指令 |
| `分类` | `categories` | 查看纯文本分类列表 |
| `重载帮助` | `reload help` | 重新加载配置文件和清除缓存 |

## 配置

资源目录：`data/kovi-plugin-help-center/*`

> 首次运行时自动生成。

### `config.toml` - 核心配置

你可以自定义标题、配色以及插件的分类归属。

```toml
# 菜单标题
title = "📚 帮助中心"
subtitle = "Kovi Bot Command Reference"
footer = "Powered by Kovi Framework"

# 触发指令
triggers = ["help", "帮助", "菜单"]

# 主题配置 (支持 CSS 颜色值)
[theme]
primary = "#6366f1"       # 主色调
bg_start = "#e0e7ff"      # 背景渐变起
bg_end = "#fdf4ff"        # 背景渐变止
card_opacity = 0.85       # 卡片透明度

# 分类配置
[[category]]
name = "🤖 基础功能"
icon = "📦"
# 插件列表
[[category.plugins]]
name = "Kovi 核心"
desc = "机器人基础管理功能"
commands = ["登录", "重启", "状态"]
icon = "⚡"

[[category]]
name = "🎮 娱乐插件"
icon = "🎮"
[[category.plugins]]
name = "今日运势"
desc = "查看今天的运气如何"
commands = ["jrrp", "运势"]
icon = "🎲"
```

## 常见问题

**Q: 图片生成速度慢？**
A: 插件首次生成图片时需要启动浏览器内核进行渲染（约 1-2 秒），生成后会自动缓存图片。后续访问相同内容的菜单将直接发送缓存图片，速度极快。

**Q: 修改了配置没生效？**
A: 请发送 `重载帮助` 指令，插件会重新读取配置文件并清理旧缓存。

## 致谢

- [Kovi](https://kovi.threkork.com/)
- [Tera](https://keats.github.io/tera/)
- [cdp-html-shot](https://crates.io/crates/cdp-html-shot)

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>
