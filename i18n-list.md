# Sea Lantern i18n 变量列表

## 通用 (common)

- common.app_name
- common.about
- common.click_to_refresh
- common.close_modal
- common.close_notification
- common.config_edit
- common.console
- common.create_server
- common.enter_command
- common.import_settings
- common.loading
- common.no_json
- common.open_folder
- common.quote_text
- common.search
- common.search_options
- common.select
- common.select_server
- common.settings
- common.unknown
- common.update_title

## 首页 (home)

- home.delete_cancel
- home.delete_confirm
- home.delete_confirm_message
- home.delete_error
- home.delete_input_placeholder
- home.detail_view
- home.error
- home.gauge_view
- home.no_servers
- home.running
- home.start
- home.starting
- home.stopped
- home.stopping
- home.stop
- home.system_resources
- home.title

## 侧边栏 (sidebar)

- sidebar.collapse_btn

## 头部 (header)

- header.english

## 创建服务器 (create)

- create.default_port_placeholder
- create.java_env
- create.java_found
- create.java_manual
- create.java_path
- create.java_scan
- create.max_memory
- create.min_memory
- create.select_java
- create.server_name
- create.title

## 控制台 (console)

- console.add_custom_command
- console.enter_command_content
- console.enter_command_name

## 配置 (config)

- config.default
- config.search
- config.properties.* (动态生成)

## 玩家 (players)

- players.ban
- players.ban_reason
- players.ban_reason_placeholder
- players.banned
- players.empty
- players.level
- players.ops
- players.player_id
- players.player_name
- players.whitelist

## 设置 (settings)

- settings.acrylic
- settings.acrylic_desc
- settings.acrylic_not_supported
- settings.animated_image
- settings.appearance
- settings.appearance_desc
- settings.background
- settings.background_desc
- settings.background_size
- settings.background_size_desc
- settings.background_size_options.auto
- settings.background_size_options.contain
- settings.background_size_options.cover
- settings.background_size_options.fill
- settings.blur
- settings.blur_desc
- settings.brightness
- settings.brightness_desc
- settings.close_action_ask
- settings.close_action_close
- settings.close_action_desc
- settings.close_action_minimize
- settings.color_alpha
- settings.color_editing
- settings.color_editing_desc
- settings.color_hue
- settings.color_lightness
- settings.color_options.custom
- settings.color_options.default
- settings.color_options.forest
- settings.color_options.midnight
- settings.color_options.ocean
- settings.color_options.rose
- settings.color_options.sunset
- settings.color_plan
- settings.color_plan_desc
- settings.color_saturation
- settings.color_theme
- settings.color_theme_desc
- settings.confirm
- settings.confirm_import
- settings.confirm_reset
- settings.cancel
- settings.delete_confirm_message
- settings.default_port
- settings.font_family
- settings.font_family_default
- settings.font_family_desc
- settings.font_size
- settings.font_size_desc
- settings.general
- settings.general_desc
- settings.import_desc
- settings.import_placeholder
- settings.import_settings
- settings.import_success
- settings.input_color_placeholder
- settings.loading
- settings.loading_preview
- settings.no_json
- settings.opacity
- settings.opacity_desc
- settings.personalize
- settings.personalize_page_import_export
- settings.pick_image
- settings.primary_background_color
- settings.primary_color
- settings.primary_text_color
- settings.remove
- settings.replace_image
- settings.reset_confirm
- settings.reset_desc
- settings.reset_success
- settings.save
- settings.secondary_background_color
- settings.secondary_color
- settings.secondary_text_color
- settings.server_defaults
- settings.server_defaults_desc
- settings.spawn_monsters
- settings.theme
- settings.theme_desc
- settings.theme_options.auto
- settings.theme_options.dark
- settings.theme_options.light
- settings.tertiary_background_color
- settings.title

## 关于 (about)

- about.contribute_ways
- about.open_link_failed
- about.project_info
- about.update_check_failed
- about.update_downloading
- about.update_running_warning_title
- about.update_title

## 托盘 (tray)

- tray.minimize
- tray.quit
- tray.show

## 动态 i18n 变量使用情况

### 带参数的变量

- home.delete_confirm_message: 带 server 参数
- create.java_found: 带 count 参数

### 动态生成的键名

- config.properties.*: 根据配置项键名动态生成，完整列表如下：
  - config.properties.accepts-transfers: 接受传输
  - config.properties.allow-flight: 允许飞行
  - config.properties.allow-nether: 允许下界
  - config.properties.broadcast-console-to-ops: 向OP广播控制台消息
  - config.properties.broadcast-rcon-to-ops: 向OP广播RCON消息
  - config.properties.bug-report-link: Bug报告链接
  - config.properties.difficulty: 游戏难度
  - config.properties.enable-command-block: 启用命令方块
  - config.properties.enable-jmx-monitoring: 启用JMX监控
  - config.properties.enable-query: 启用Query协议
  - config.properties.enable-rcon: 启用RCON远程控制
  - config.properties.enable-status: 启用服务器列表状态
  - config.properties.enforce-secure-profile: 强制安全配置文件
  - config.properties.enforce-whitelist: 强制白名单
  - config.properties.entity-broadcast-range-percentage: 实体广播范围百分比
  - config.properties.force-gamemode: 强制游戏模式
  - config.properties.function-permission-level: 函数权限级别
  - config.properties.gamemode: 默认游戏模式
  - config.properties.generate-structures: 生成结构
  - config.properties.generator-settings: 生成器设置
  - config.properties.hardcore: 极限模式
  - config.properties.hide-online-players: 隐藏在线玩家
  - config.properties.initial-disabled-packs: 初始禁用的数据包
  - config.properties.initial-enabled-packs: 初始启用的数据包
  - config.properties.level-name: 世界名称
  - config.properties.level-seed: 世界种子
  - config.properties.level-type: 世界类型
  - config.properties.log-ips: 记录IP地址
  - config.properties.max-chained-neighbor-updates: 最大连锁邻居更新
  - config.properties.max-players: 最大玩家数
  - config.properties.max-tick-time: 最大tick时间
  - config.properties.max-world-size: 最大世界大小
  - config.properties.motd: 服务器描述
  - config.properties.network-compression-threshold: 网络压缩阈值
  - config.properties.online-mode: 正版验证
  - config.properties.op-permission-level: OP权限级别
  - config.properties.pause-when-empty-seconds: 空服务器暂停秒数
  - config.properties.player-idle-timeout: 玩家空闲超时
  - config.properties.prevent-proxy-connections: 防止代理连接
  - config.properties.pvp: 允许PVP
  - config.properties.query.port: Query端口
  - config.properties.rate-limit: 速率限制
  - config.properties.rcon.password: RCON密码
  - config.properties.rcon.port: RCON端口
  - config.properties.region-file-compression: 区域文件压缩
  - config.properties.require-resource-pack: 要求资源包
  - config.properties.resource-pack: 资源包URL
  - config.properties.resource-pack-id: 资源包ID
  - config.properties.resource-pack-prompt: 资源包提示
  - config.properties.resource-pack-sha1: 资源包SHA1哈希
  - config.properties.server-ip: 服务器IP
  - config.properties.server-port: 服务器端口
  - config.properties.simulation-distance: 模拟距离
  - config.properties.spawn-monsters: 生成怪物
  - config.properties.spawn-protection: 出生点保护半径
  - config.properties.sync-chunk-writes: 同步区块写入
  - config.properties.text-filtering-config: 文本过滤配置
  - config.properties.text-filtering-version: 文本过滤版本
  - config.properties.use-native-transport: 使用原生传输
  - config.properties.view-distance: 视距
  - config.properties.white-list: 白名单

