//! # kovi-plugin-help-center
//!
//! 高颜值、高性能的帮助菜单插件，基于 HTML/CSS 渲染。
//!
//! ## 功能特性
//! - 🎨 玻璃拟态 UI 设计，支持自定义主题
//! - 📁 分类展示，层次清晰
//! - 🔍 支持按名称/指令搜索
//! - 💾 智能缓存，配置变更自动刷新
//! - 🔄 支持热重载配置

// ============================================================================
//                              配置模块
// ============================================================================
mod config {
    use kovi::toml;
    use kovi::utils::load_toml_data;
    use serde::{Deserialize, Serialize};
    use std::hash::{Hash, Hasher};
    use std::path::PathBuf;
    use std::sync::{Arc, OnceLock, RwLock};

    pub static CONFIG: OnceLock<Arc<RwLock<Config>>> = OnceLock::new();

    const DEFAULT_CONFIG: &str = include_str!("default_config.toml");

    #[derive(Debug, Serialize, Deserialize, Clone, Hash)]
    pub struct PluginItem {
        pub name: String,
        pub desc: String,
        #[serde(default)]
        pub commands: Vec<String>,
        /// 可选的图标 emoji
        #[serde(default)]
        pub icon: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, Hash)]
    pub struct Category {
        pub name: String,
        /// 分类图标
        #[serde(default)]
        pub icon: String,
        /// 分类颜色（可选，覆盖主题色）
        #[serde(default)]
        pub color: Option<String>,
        #[serde(default)]
        pub plugins: Vec<PluginItem>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Theme {
        /// 主色调
        #[serde(default = "default_primary")]
        pub primary: String,
        /// 背景渐变起始色
        #[serde(default = "default_bg_start")]
        pub bg_start: String,
        /// 背景渐变结束色
        #[serde(default = "default_bg_end")]
        pub bg_end: String,
        /// 卡片背景透明度 0.0-1.0
        #[serde(default = "default_card_opacity")]
        pub card_opacity: f32,
    }

    fn default_primary() -> String {
        "#6366f1".into()
    }
    fn default_bg_start() -> String {
        "#e0e7ff".into()
    }
    fn default_bg_end() -> String {
        "#fdf4ff".into()
    }
    fn default_card_opacity() -> f32 {
        0.85
    }

    impl Default for Theme {
        fn default() -> Self {
            Self {
                primary: default_primary(),
                bg_start: default_bg_start(),
                bg_end: default_bg_end(),
                card_opacity: default_card_opacity(),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Config {
        /// 帮助菜单标题
        #[serde(default = "default_title")]
        pub title: String,
        /// 副标题
        #[serde(default)]
        pub subtitle: Option<String>,
        /// 底部文字
        #[serde(default = "default_footer")]
        pub footer: String,
        /// 主题配置
        #[serde(default)]
        pub theme: Theme,
        /// 分类列表
        #[serde(default)]
        pub category: Vec<Category>,
        /// 触发词列表（可自定义）
        #[serde(default = "default_triggers")]
        pub triggers: Vec<String>,
        /// 配置文件路径（内部使用）
        #[serde(skip)]
        pub config_path: PathBuf,
    }

    fn default_title() -> String {
        "📚 帮助中心".into()
    }
    fn default_footer() -> String {
        "Powered by Kovi Framework".into()
    }
    fn default_triggers() -> Vec<String> {
        vec![
            "help".into(),
            "帮助".into(),
            "菜单".into(),
            "menu".into(),
            "指令".into(),
            "功能".into(),
        ]
    }

    impl Config {
        pub fn load(data_dir: &PathBuf) -> Arc<RwLock<Self>> {
            if !data_dir.exists() {
                std::fs::create_dir_all(data_dir).expect("Failed to create data directory");
            }
            let config_path = data_dir.join("config.toml");

            let default: Config =
                toml::from_str(DEFAULT_CONFIG).expect("Default config parse error");

            let mut config = load_toml_data(default, config_path.clone()).unwrap_or_else(|e| {
                kovi::log::warn!("配置加载失败，使用默认配置: {}", e);
                toml::from_str(DEFAULT_CONFIG).unwrap()
            });

            config.config_path = config_path;
            Arc::new(RwLock::new(config))
        }

        /// 重新加载配置
        pub fn reload(&mut self) -> Result<(), String> {
            let new_config: Config = {
                let content = std::fs::read_to_string(&self.config_path)
                    .map_err(|e| format!("读取配置失败: {}", e))?;
                toml::from_str(&content).map_err(|e| format!("解析配置失败: {}", e))?
            };

            self.title = new_config.title;
            self.subtitle = new_config.subtitle;
            self.footer = new_config.footer;
            self.theme = new_config.theme;
            self.category = new_config.category;
            self.triggers = new_config.triggers;

            Ok(())
        }

        /// 计算配置哈希用于缓存
        pub fn content_hash(&self) -> u64 {
            use std::collections::hash_map::DefaultHasher;
            let mut hasher = DefaultHasher::new();
            self.title.hash(&mut hasher);
            self.subtitle.hash(&mut hasher);
            format!("{:?}", self.theme).hash(&mut hasher);
            for cat in &self.category {
                cat.hash(&mut hasher);
            }
            hasher.finish()
        }

        /// 搜索指令/插件
        pub fn search(&self, keyword: &str) -> Vec<SearchResult> {
            let kw = keyword.to_lowercase();
            let mut results = Vec::new();

            for cat in &self.category {
                for plugin in &cat.plugins {
                    // 匹配插件名
                    if plugin.name.to_lowercase().contains(&kw) {
                        results.push(SearchResult {
                            category: cat.name.clone(),
                            plugin: plugin.name.clone(),
                            desc: plugin.desc.clone(),
                            matched_cmd: None,
                        });
                        continue;
                    }
                    // 匹配指令
                    for cmd in &plugin.commands {
                        if cmd.to_lowercase().contains(&kw) {
                            results.push(SearchResult {
                                category: cat.name.clone(),
                                plugin: plugin.name.clone(),
                                desc: plugin.desc.clone(),
                                matched_cmd: Some(cmd.clone()),
                            });
                            break;
                        }
                    }
                }
            }
            results
        }

        /// 获取分类列表
        pub fn category_names(&self) -> Vec<String> {
            self.category.iter().map(|c| c.name.clone()).collect()
        }
    }

    #[derive(Debug, Clone)]
    pub struct SearchResult {
        pub category: String,
        pub plugin: String,
        pub desc: String,
        pub matched_cmd: Option<String>,
    }
}

// ============================================================================
//                              渲染模块
// ============================================================================
mod render {
    use super::config::Config;
    use anyhow::Result;
    use cdp_html_shot::{Browser, CaptureOptions, Viewport};
    use kovi::tokio;
    use std::path::Path;
    use tera::{Context, Tera};

    /// 现代化玻璃拟态风格模板
    const HTML_TEMPLATE: &str = r##"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }

        :root {
            --primary: {{ theme.primary }};
            --bg-start: {{ theme.bg_start }};
            --bg-end: {{ theme.bg_end }};
            --card-opacity: {{ theme.card_opacity }};
        }

        body {
            font-family: 'HarmonyOS Sans SC', 'PingFang SC', 'Microsoft YaHei', sans-serif;
            background: linear-gradient(135deg, var(--bg-start) 0%, var(--bg-end) 100%);
            min-height: 100vh;
            padding: 32px;
            width: 900px;
        }

        .container {
            display: flex;
            flex-direction: column;
            gap: 24px;
        }

        /* 头部区域 */
        .header {
            text-align: center;
            padding: 24px 0;
            position: relative;
        }

        .header::before {
            content: '';
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            width: 200px;
            height: 200px;
            background: radial-gradient(circle, var(--primary) 0%, transparent 70%);
            opacity: 0.15;
            border-radius: 50%;
            z-index: 0;
        }

        .title {
            font-size: 36px;
            font-weight: 800;
            background: linear-gradient(135deg, var(--primary) 0%, #a855f7 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            position: relative;
            z-index: 1;
            letter-spacing: 2px;
        }

        .subtitle {
            font-size: 14px;
            color: #64748b;
            margin-top: 8px;
            letter-spacing: 4px;
            text-transform: uppercase;
        }

        /* 分类区域 */
        .category-section {
            background: rgba(255, 255, 255, var(--card-opacity));
            backdrop-filter: blur(20px);
            -webkit-backdrop-filter: blur(20px);
            border-radius: 20px;
            padding: 24px;
            border: 1px solid rgba(255, 255, 255, 0.5);
            box-shadow:
                0 4px 24px rgba(0, 0, 0, 0.06),
                inset 0 1px 0 rgba(255, 255, 255, 0.8);
        }

        .category-header {
            display: flex;
            align-items: center;
            gap: 12px;
            margin-bottom: 20px;
            padding-bottom: 12px;
            border-bottom: 2px solid rgba(99, 102, 241, 0.1);
        }

        .category-icon {
            font-size: 24px;
            width: 44px;
            height: 44px;
            display: flex;
            align-items: center;
            justify-content: center;
            background: linear-gradient(135deg, var(--primary) 0%, #a855f7 100%);
            border-radius: 12px;
            box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
        }

        .category-name {
            font-size: 20px;
            font-weight: 700;
            color: #1e293b;
        }

        .category-count {
            font-size: 12px;
            color: #94a3b8;
            background: #f1f5f9;
            padding: 4px 10px;
            border-radius: 20px;
            margin-left: auto;
        }

        /* 插件网格 */
        .plugins-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 16px;
        }

        .plugin-card {
            background: rgba(255, 255, 255, 0.7);
            border-radius: 14px;
            padding: 16px;
            border: 1px solid rgba(255, 255, 255, 0.8);
            transition: all 0.2s ease;
        }

        .plugin-header {
            display: flex;
            align-items: center;
            gap: 10px;
            margin-bottom: 12px;
        }

        .plugin-icon {
            font-size: 20px;
        }

        .plugin-info {
            flex: 1;
            min-width: 0;
        }

        .plugin-name {
            font-size: 15px;
            font-weight: 600;
            color: #334155;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }

        .plugin-desc {
            font-size: 12px;
            color: #64748b;
            margin-top: 2px;
        }

        /* 指令标签 */
        .commands {
            display: flex;
            flex-wrap: wrap;
            gap: 6px;
        }

        .cmd-tag {
            font-family: 'JetBrains Mono', 'Fira Code', monospace;
            font-size: 11px;
            padding: 5px 10px;
            background: linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(168, 85, 247, 0.1) 100%);
            color: var(--primary);
            border-radius: 8px;
            font-weight: 500;
            border: 1px solid rgba(99, 102, 241, 0.15);
        }

        /* 底部 */
        .footer {
            text-align: center;
            padding: 20px 0 8px;
            color: #94a3b8;
            font-size: 12px;
        }

        .footer-divider {
            width: 60px;
            height: 3px;
            background: linear-gradient(90deg, transparent, var(--primary), transparent);
            margin: 0 auto 12px;
            border-radius: 2px;
            opacity: 0.5;
        }

        .tip {
            margin-top: 8px;
            font-size: 11px;
            color: #cbd5e1;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1 class="title">{{ title }}</h1>
            {% if subtitle %}
            <div class="subtitle">{{ subtitle }}</div>
            {% else %}
            <div class="subtitle">Command Reference</div>
            {% endif %}
        </div>

        {% for cat in category %}
        <div class="category-section">
            <div class="category-header">
                <div class="category-icon">{{ cat.icon | default(value="📦") }}</div>
                <span class="category-name">{{ cat.name }}</span>
                <span class="category-count">{{ cat.plugins | length }} 个插件</span>
            </div>
            <div class="plugins-grid">
                {% for plugin in cat.plugins %}
                <div class="plugin-card">
                    <div class="plugin-header">
                        <span class="plugin-icon">{{ plugin.icon | default(value="⚡") }}</span>
                        <div class="plugin-info">
                            <div class="plugin-name">{{ plugin.name }}</div>
                            <div class="plugin-desc">{{ plugin.desc }}</div>
                        </div>
                    </div>
                    <div class="commands">
                        {% for cmd in plugin.commands %}
                        <span class="cmd-tag">{{ cmd }}</span>
                        {% endfor %}
                    </div>
                </div>
                {% endfor %}
            </div>
        </div>
        {% endfor %}

        <div class="footer">
            <div class="footer-divider"></div>
            <div>{{ footer }}</div>
            <div class="tip">💡 发送「帮助 关键词」可搜索指令</div>
        </div>
    </div>
</body>
</html>
"##;

    /// 生成 HTML
    pub fn build_html(config: &Config) -> Result<String> {
        let mut tera = Tera::default();
        tera.add_raw_template("help", HTML_TEMPLATE)?;
        let ctx = Context::from_serialize(config)?;
        Ok(tera.render("help", &ctx)?)
    }

    /// 渲染为 PNG 图片
    pub async fn render_to_png(html: &str, output: &Path) -> Result<()> {
        let browser = Browser::instance().await;

        let opts = CaptureOptions::new()
            .with_viewport(Viewport::new(932, 100).with_device_scale_factor(2.0))
            .with_quality(92)
            .with_full_page(true);

        let base64 = browser
            .capture_html_with_options(html, "body", opts)
            .await?;

        let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64)?;

        tokio::fs::write(output, bytes).await?;
        Ok(())
    }
}

// ============================================================================
//                              缓存管理
// ============================================================================
mod cache {
    use std::path::{Path, PathBuf};

    use kovi::tokio;

    /// 获取缓存文件路径
    pub fn get_cache_path(data_dir: &Path, hash: u64) -> PathBuf {
        data_dir.join(format!("help_{:016x}.png", hash))
    }

    /// 检查缓存是否存在且有效
    pub fn is_valid(path: &Path) -> bool {
        path.exists() && path.metadata().map(|m| m.len() > 0).unwrap_or(false)
    }

    /// 清理旧缓存（保留当前 hash 的文件）
    pub async fn cleanup(data_dir: &Path, current_hash: u64) {
        let current_name = format!("help_{:016x}.png", current_hash);

        let Ok(mut entries) = tokio::fs::read_dir(data_dir).await else {
            return;
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("help_") && name.ends_with(".png") && name != current_name {
                let _ = tokio::fs::remove_file(entry.path()).await;
            }
        }
    }
}

// ============================================================================
//                              消息处理
// ============================================================================
mod handler {
    use super::{cache, config, render};
    use kovi::{Message, MsgEvent, log, tokio};
    use std::path::Path;
    use std::sync::{Arc, RwLock};

    /// 处理帮助指令
    pub async fn handle_help(
        event: &Arc<MsgEvent>,
        config_lock: &Arc<RwLock<config::Config>>,
        data_dir: &Path,
    ) {
        let (config, hash) = {
            let cfg = config_lock.read().unwrap();
            (cfg.clone(), cfg.content_hash())
        };

        let cache_path = cache::get_cache_path(data_dir, hash);

        // 检查缓存
        if !cache::is_valid(&cache_path) {
            event.reply("🎨 正在生成帮助菜单...");

            // 生成 HTML
            let html = match render::build_html(&config) {
                Ok(h) => h,
                Err(e) => {
                    log::error!("HTML 生成失败: {}", e);
                    event.reply("❌ 菜单生成失败，请稍后重试");
                    return;
                }
            };

            // 渲染图片
            if let Err(e) = render::render_to_png(&html, &cache_path).await {
                log::error!("图片渲染失败: {}", e);
                event.reply("❌ 图片渲染失败");
                return;
            }

            // 异步清理旧缓存
            let dir = data_dir.to_path_buf();
            tokio::spawn(async move {
                cache::cleanup(&dir, hash).await;
            });
        }

        // 发送图片
        let path_str = cache_path.to_string_lossy().replace('\\', "/");
        let msg = Message::new()
            .add_reply(event.message_id)
            .add_image(&format!("file:///{}", path_str));
        event.reply(msg);
    }

    /// 处理搜索指令
    pub fn handle_search(
        event: &Arc<MsgEvent>,
        keyword: &str,
        config_lock: &Arc<RwLock<config::Config>>,
    ) {
        let results = {
            let cfg = config_lock.read().unwrap();
            cfg.search(keyword)
        };

        if results.is_empty() {
            event.reply(format!("🔍 未找到与「{}」相关的指令", keyword));
            return;
        }

        let mut msg = format!("🔍 搜索「{}」找到 {} 条结果：\n\n", keyword, results.len());

        for (i, r) in results.iter().take(8).enumerate() {
            msg.push_str(&format!(
                "{}. 【{}】{}\n   📝 {}\n",
                i + 1,
                r.category,
                r.plugin,
                r.desc
            ));
            if let Some(cmd) = &r.matched_cmd {
                msg.push_str(&format!("   🎯 匹配: {}\n", cmd));
            }
            msg.push('\n');
        }

        if results.len() > 8 {
            msg.push_str(&format!("...还有 {} 条结果", results.len() - 8));
        }

        event.reply(msg.trim());
    }

    /// 处理配置重载
    pub fn handle_reload(
        event: &Arc<MsgEvent>,
        config_lock: &Arc<RwLock<config::Config>>,
        data_dir: &Path,
    ) {
        let result = {
            let mut cfg = config_lock.write().unwrap();
            cfg.reload()
        };

        match result {
            Ok(()) => {
                // 清除所有缓存
                if let Ok(entries) = std::fs::read_dir(data_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("help_") && name.ends_with(".png") {
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
                event.reply("✅ 配置重载成功！下次查看帮助将使用新配置");
            }
            Err(e) => {
                event.reply(format!("❌ 配置重载失败: {}", e));
            }
        }
    }

    /// 处理分类列表查询
    pub fn handle_categories(event: &Arc<MsgEvent>, config_lock: &Arc<RwLock<config::Config>>) {
        let names = {
            let cfg = config_lock.read().unwrap();
            cfg.category_names()
        };

        if names.is_empty() {
            event.reply("📂 暂无分类配置");
            return;
        }

        let mut msg = String::from("📂 当前分类列表：\n\n");
        for (i, name) in names.iter().enumerate() {
            msg.push_str(&format!("  {}. {}\n", i + 1, name));
        }
        msg.push_str("\n💡 发送「帮助 分类名」可搜索该分类下的指令");
        event.reply(msg);
    }
}

// ============================================================================
//                              插件入口
// ============================================================================
use kovi::{PluginBuilder, tokio};

#[kovi::plugin]
async fn main() {
    let bot = PluginBuilder::get_runtime_bot();
    let data_dir = bot.get_data_path();

    // 初始化配置
    let config_lock = config::Config::load(&data_dir);
    config::CONFIG.set(config_lock.clone()).ok();

    // 预热浏览器（后台异步）
    tokio::spawn(async {
        cdp_html_shot::Browser::instance().await;
        kovi::log::info!("[help-center] 浏览器预热完成");
    });

    // 消息处理
    PluginBuilder::on_msg(move |event| {
        let config_lock = config_lock.clone();
        let data_dir = data_dir.clone();

        async move {
            let text = match event.borrow_text() {
                Some(t) => t.trim(),
                None => return,
            };

            // 获取触发词列表
            let triggers: Vec<String> = {
                let cfg = config_lock.read().unwrap();
                cfg.triggers.clone()
            };

            let text_lower = text.to_lowercase();

            // 1. 检查是否是搜索指令: "帮助 xxx" / "搜索 xxx"
            for prefix in ["帮助 ", "help ", "搜索 ", "search "] {
                if let Some(keyword) = text_lower.strip_prefix(prefix) {
                    let keyword = keyword.trim();
                    if !keyword.is_empty() {
                        handler::handle_search(&event, keyword, &config_lock);
                        return;
                    }
                }
            }

            // 2. 检查是否是重载指令（可添加权限检查）
            if matches!(text_lower.as_str(), "重载帮助" | "reload help" | "帮助重载") {
                handler::handle_reload(&event, &config_lock, &data_dir);
                return;
            }

            // 3. 检查是否是查看分类指令
            if matches!(text_lower.as_str(), "分类" | "分类列表" | "categories") {
                handler::handle_categories(&event, &config_lock);
                return;
            }

            // 4. 检查是否是帮助指令
            if triggers.iter().any(|t| t.to_lowercase() == text_lower) {
                handler::handle_help(&event, &config_lock, &data_dir).await;
            }
        }
    });
}
