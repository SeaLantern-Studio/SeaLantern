# Minecraft 命令树（Java Edition 完整版）

## 基础命令（一级，无需 OP）

```
/help [页码|命令名]         — 显示帮助信息
/? [页码|命令名]            — /help 别名
/list                       — 列出在线玩家
/me <动作文本>              — 展示动作（如 *玩家 正在挖掘*）
/msg <玩家> <消息>          — 私信
/tell <玩家> <消息>         — /msg 别名
/w <玩家> <消息>            — /msg 别名
/seed                       — 查看世界种子
/trigger <目标>             — 触发触发器
/tm <消息>                  — 团队聊天
/teammsg <消息>             — 团队聊天
/random [范围]              — 随机数
```

## 管理类命令（需 OP level 2+）

### 玩家/服务器管理

```
/op <玩家>                  — 赋予管理员
/deop <玩家>                — 撤销管理员
/ban <玩家> [原因]          — 封禁玩家
/ban-ip <IP> [原因]         — 封禁 IP
/banlist [ips|players]      — 显示封禁列表
/pardon <玩家>              — 解封玩家
/pardon-ip <IP>             — 解封 IP
/kick <玩家> [原因]         — 踢出玩家
/whitelist add|remove|list|on|off <玩家>
                            — 白名单管理
/setidletimeout <分钟>      — 设置挂机踢出时间
/stop                       — 关闭服务器
/save-all                   — 强制保存
/save-on                    — 开启自动保存
/save-off                   — 关闭自动保存
/publish                    — 对局域网开放
/transfer <玩家> <服务器>   — 转移玩家到其他服务器
```

### 游戏模式

```
/gamemode <模式> [玩家]     — 设置游戏模式
  survival / 0              — 生存模式
  creative / 1              — 创造模式
  adventure / 2             — 冒险模式
  spectator / 3             — 旁观模式
/defaultgamemode <模式>     — 设置默认游戏模式
```

### 难度与时间天气

```
/difficulty <难度>          — 设置难度
  peaceful / 0              — 和平
  easy / 1                  — 简单
  normal / 2                — 普通
  hard / 3                  — 困难

/time set <值|day|night|noon>
  day                       — 设置为白天（1000）
  night                     — 设置为夜晚（13000）
  noon                      — 设置为正午（6000）
  <数值>                    — 设置为指定时间刻
/time add <数值>            — 增加时间
/time query daytime|gametime— 查询时间

/weather <clear|rain|thunder> [持续时间]
                            — 设置天气
/toggledownfall             — 切换雨雪
```

### 游戏规则 (gamerule)

```
/gamerule <规则名> <true|false|数值>

常用规则：
  keepInventory             — 死亡不掉落
  mobGriefing               — 生物破坏方块
  naturalRegeneration       — 自然回血
  doFireTick                — 火焰蔓延
  doMobSpawning             — 生物生成
  doDaylightCycle           — 昼夜循环
  doWeatherCycle            — 天气循环
  showDeathMessages         — 显示死亡信息
  showCoordinates           — 显示坐标
  commandBlockOutput        — 命令方块输出
  tntExplodes               — TNT 爆炸
  fallDamage                — 摔落伤害
  fireDamage                — 火焰伤害
  drowningDamage            — 溺水伤害
  doMobLoot                 — 生物掉落
  doTileDrops               — 方块掉落
  announceAdvancements      — 公告进度
  maxCommandChainLength     — 最大命令链长度
  randomTickSpeed           — 随机刻速度
  playersSleepingPercentage — 跳过夜晚所需玩家百分比
```

### 传送与定位

```
/tp <目标> [目的地]        — 传送
/teleport <目标> [目的地]  — /tp 别名
/spreadplayers <x> <z> <间距> <最大范围> [队伍|玩家]
                            — 随机传送
/locate structure|biome|poi <类型>
                            — 定位结构/生物群系
/locatebiome <生物群系>    — 定位生物群系
/setworldspawn [x y z]     — 设置世界出生点
/spawnpoint [玩家] [x y z] — 设置玩家出生点
/spectate [目标] [玩家]    — 旁观模式观看实体
/ride <目标> mount|dismount [坐骑]
                            — 骑乘实体
```

### 物品管理

```
/give <玩家> <物品> [数量] [NBT]
                            — 给予物品
/clear [玩家] [物品] [数量]
                            — 清除物品
/item replace entity <目标> <槽位> with <物品>
                            — 替换物品  (1.17+)
/enchant <玩家> <附魔> [等级]
                            — 附魔
/recipe give|take [玩家] [配方]
                            — 配方管理
/drop <目标>                — 丢弃物品
```

### 状态效果

```
/effect give <目标> <效果> [秒数] [倍率] [粒子]
                            — 给予效果
/effect clear <目标> [效果] — 清除效果
```

### 经验

```
/xp add|set|query <目标> <数量> [points|levels]
                            — 经验管理
/experience               — /xp 别名
```

### 声音与粒子

```
/playsound <声音> <来源> [玩家] [x y z] [音量] [音调]
                            — 播放声音
/stopsound [玩家] [来源] [声音]
                            — 停止声音
/particle <粒子> [位置] [数量] [速度]
                            — 生成粒子
/title <玩家> clear|reset|title|subtitle|actionbar <文本>
                            — 标题/字幕/操作栏
```

### 实体控制

```
/summon <实体> [x y z] [NBT]
                            — 生成实体
/kill [目标]               — 杀死实体
/damage <目标> <伤害> [原因]
                            — 造成伤害  (1.20+)
/ride <目标> mount|dismount
                            — 骑乘
/rotate <目标> <角度>      — 旋转实体
/swing <目标> [手]         — 挥舞手臂
/tag <目标> add|remove|list <标签>
                            — 标签管理
/team add|remove|join|leave|modify|empty|list
                            — 队伍管理
```

### 方块与世界

```
/setblock <x y z> <方块> [destroy|replace|keep]
                            — 放置方块
/fill <x1 y1 z1> <x2 y2 z2> <方块> [模式]
                            — 填充方块区域
/clone <x1 y1 z1> <x2 y2 z2> <x y z> [模式]
                            — 复制方块区域
/fillbiome <x1 y1 z1> <x2 y2 z2> <生物群系>
                            — 填充生物群系  (1.19+)
/place feature|jigsaw|structure|template <类型>
                            — 放置结构
/loot spawn|replace|give|insert <来源>
                            — 战利品操作
/worldborder <操作> <参数> — 世界边界管理
  add <距离> [时间]         — 扩大/缩小
  set <距离> [时间]         — 设置大小
  center <x z>              — 设置中心
  damage <伤害>             — 设置边界伤害
  warning <距离>            — 设置警告距离
```

### 数据与NBT

```
/data get|merge|modify|remove <目标> [NBT路径]
                            — 数据操作
/attribute <目标> <属性> get|set|modifier
                            — 属性操作
```

### 进度与计分板

```
/advancement grant|revoke <目标> <条件>
                            — 进度管理
/scoreboard objectives|players <操作> <参数>
                            — 计分板管理
```

### 函数与数据包

```
/function <名称>            — 运行函数
/schedule <函数> <时间>    — 延迟执行函数
/reload                    — 重新加载数据包
/datapack enable|disable|list <名称>
                            — 数据包管理
```

### 调试命令

```
/debug start|stop          — 调试分析
/perf                      — 性能分析 (1.19+)
/jfr                       — JFR 分析
/tick freeze|step|unfreeze|rate|query
                            — 控制游戏刻速率
```

### 命令方块

```
/deop                       — 撤销管理员
/tellraw <玩家> <JSON消息> — JSON 消息
```

---

## 命令树数据结构（供代码参考）

```typescript
const commandTree: Record<string, string[]> = {
  // 基础（无子命令）
  help: [],
  list: [],
  me: [],
  msg: [],
  tell: [],
  w: [],
  seed: [],
  stop: [],
  saveAll: [],
  reload: [],
  publish: [],
  tps: [],
  plugins: [],
  version: [],
  kill: [],

  // 管理员
  op: [],
  deop: [],
  ban: [],
  pardon: [],
  kick: [],
  whitelist: ["add", "remove", "list"],
  banlist: ["players", "ips"],

  // 游戏模式
  gamemode: ["survival", "creative", "adventure", "spectator"],
  defaultgamemode: ["survival", "creative", "adventure", "spectator"],

  // 难度
  difficulty: ["peaceful", "easy", "normal", "hard"],

  // 时间
  time: ["set", "add", "query"],

  // 天气
  weather: ["clear", "rain", "thunder"],

  // 基础操作
  tp: [],
  teleport: [],
  give: [],
  say: [],
  clear: [],
  enchant: [],
  effect: ["give", "clear"],
  xp: ["add", "set", "query"],
  experience: [],
  summon: [],

  // 游戏规则
  gamerule: [
    "keepInventory",
    "mobGriefing",
    "naturalRegeneration",
    "doFireTick",
    "doDaylightCycle",
    "doWeatherCycle",
    "doMobSpawning",
    "doMobLoot",
    "doTileDrops",
    "fallDamage",
    "fireDamage",
    "drowningDamage",
    "showDeathMessages",
    "showCoordinates",
    "commandBlockOutput",
    "tntExplodes",
    "announceAdvancements",
    "playersSleepingPercentage",
    "randomTickSpeed",
    "maxCommandChainLength",
  ],

  // 定位
  locate: ["structure", "biome", "poi"],
  locatebiome: [],
  setworldspawn: [],
  spawnpoint: [],

  // 方块操作
  setblock: [],
  fill: [],
  clone: [],
  fillbiome: [],

  // 数据
  data: ["get", "merge", "modify", "remove"],
  attribute: ["get", "set"],
  scoreboard: ["objectives", "players"],

  // 函数
  function: [],
  schedule: [],
  datapack: ["enable", "disable", "list"],

  // 标签/队伍
  tag: ["add", "remove", "list"],
  team: ["add", "remove", "join", "leave", "list"],

  // 声音/粒子
  playsound: [],
  stopsound: [],
  particle: [],
  title: ["clear", "reset", "title", "subtitle", "actionbar"],
  tellraw: [],

  // 调试
  debug: ["start", "stop"],
  tick: ["freeze", "step", "unfreeze", "rate", "query"],
  perf: [],
  jfr: [],
  worldborder: ["add", "set", "center", "damage", "warning"],
};
```

## gamerule 值选项

```typescript
const gameruleValues: Record<string, string[]> = {
  keepInventory: ["true", "false"],
  doDaylightCycle: ["true", "false"],
  doWeatherCycle: ["true", "false"],
  doMobSpawning: ["true", "false"],
  mobGriefing: ["true", "false"],
  doFireTick: ["true", "false"],
  doMobLoot: ["true", "false"],
  doTileDrops: ["true", "false"],
  naturalRegeneration: ["true", "false"],
  fallDamage: ["true", "false"],
  fireDamage: ["true", "false"],
  drowningDamage: ["true", "false"],
  showDeathMessages: ["true", "false"],
  showCoordinates: ["true", "false"],
  commandBlockOutput: ["true", "false"],
  tntExplodes: ["true", "false"],
  announceAdvancements: ["true", "false"],
  playersSleepingPercentage: [], // 数值 0-100
  randomTickSpeed: [], // 数值
  maxCommandChainLength: [], // 数值 1+
};

// time set 的值
const timeValues = ["day", "night", "noon", "midnight"];
```
