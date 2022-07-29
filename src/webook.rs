#[derive(Debug, Clone, Copy)]
pub enum SourceType {
    Text,
    Music,
    Video,
    Picture,
    File,
}

pub struct BookSource {
    /// 地址，包括http/https
    url: String,
    /// 名称
    name: String,
    /// 分组
    group: Option<String>,
    /// 类型
    r_type: SourceType,
    /// 详情页url正则
    url_pattern: Option<String>,
    /// 是否启用
    enabled: bool,
    /// 启用okhttp CookieJAr 自动保存每次请求的cookie
    enabled_cookiejar: bool,
    /// 并发率
    concurrent_rate: Option<String>,
    /// 请求头
    header: Option<String>,
    /// 登录地址
    login_url: Option<String>,
    /// 登录UI
    login_ui: Option<String>,
    /// 登录检测js
    login_checkjs: Option<String>,
    /// 注释
    comment: Option<String>,
    /// 自定义变量说明
    variable_comment: Option<String>,
    /// 发现url
    explore_url: Option<String>,
    /// 发现规则
    rule_explore: ExploreRule,
    /// 搜索url
    search_url: Option<String>,
    /// 搜索规则
    rule_search: Option<SearchRule>,
    /// 书籍信息页规则
    rule_book_info: Option<InfoRule>,
    /// 目录页规则
    rule_toc: Option<TocRule>,
    /// 正文页规则
    rule_content: Option<ContentRule>,
}

pub struct ExploreRule {}

pub struct SearchRule {}

pub struct InfoRule {}

pub struct TocRule {}

pub struct ContentRule {}
