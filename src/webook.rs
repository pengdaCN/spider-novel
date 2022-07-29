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
    iable_comment: Option<String>,
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

pub struct ExploreRule {
    book_list: Option<String>,
    name: Option<String>,
    author: Option<String>,
    intro: Option<String>,
    kind: Option<String>,
    last_chapter: Option<String>,
    update_time: Option<String>,
    book_url: Option<String>,
    cover_url: Option<String>,
    word_count: Option<String>,
}

pub struct SearchRule {
    /// 校验关键字
    check_keyword: Option<String>,
    book_list: Option<String>,
    name: Option<String>,
    author: Option<String>,
    intro: Option<String>,
    kind: Option<String>,
    last_chapter: Option<String>,
    update_time: Option<String>,
    book_url: Option<String>,
    cover_url: Option<String>,
    word_count: Option<String>,
}

pub struct InfoRule {
    init: Option<String>,
    name: Option<String>,
    author: Option<String>,
    intro: Option<String>,
    kind: Option<String>,
    last_chapter: Option<String>,
    update_time: Option<String>,
    cover_url: Option<String>,
    toc_url: Option<String>,
    word_count: Option<String>,
    can_re_name: Option<String>,
    download_urls: Option<String>,
}

pub struct TocRule {
    pre_update_js: Option<String>,
    chapter_list: Option<String>,
    chapter_name: Option<String>,
    chapter_url: Option<String>,
    is_volume: Option<String>,
    is_vip: Option<String>,
    is_pay: Option<String>,
    update_time: Option<String>,
    next_toc_url: Option<String>,
}

pub struct ContentRule {
    content: Option<String>,
    next_content_url: Option<String>,
    web_js: Option<String>,
    source_regex: Option<String>,
    /// 替换规则
    replace_regex: Option<String>,
    /// 默认大小居中,FULL最大宽度
    image_style: Option<String>,
    /// 购买操作,js或者包含{{js}}的url
    pay_action: Option<String>,
}
