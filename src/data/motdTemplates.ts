/**
 * MOTD 可视化编辑器的内置模板
 *
 * 模板内容移植自 Snowolf（提供者：LR小さな狐の妖精）。
 * value 为 server.properties 存储格式：包含 motd= 前缀与字面 \n 换行。
 * name 通过 i18n 键 config.motd.names.{name} + 序号 渲染。
 */

export interface MotdTemplate {
  /** i18n 样式键后缀，如 classic / cyber */
  name: string;
  /** 同样式下的序号，从 0 开始 */
  index: number;
  value: string[];
}

export const MOTD_TEMPLATES: MotdTemplate[] = [
  {
    name: "classic",
    index: 0,
    value: ["motd=§aWelcome to §2MyServer§a!", "§6Survival §7• §bPvP §7• §cMinigames"],
  },
  {
    name: "classic",
    index: 1,
    value: ["motd=§c§l✦ §6§lEPIC §e§lSERVER §c§l✦", "§bJoin now for free rewards!"],
  },
  {
    name: "classic",
    index: 2,
    value: ["motd=§9■§b■§9■§b■§9■§b■§9■§b■§9■§b■§9■", "§3§lCreative Plots §f- §a§lFree Ranks"],
  },
  {
    name: "classical",
    index: 0,
    value: ["motd=§6§l✿ 星落圣域 · 长期服 ✿ §r", "§7官网:mc-xxx.top §8| §aQQ群:XXX §8| §e纯净生存"],
  },
  {
    name: "classical",
    index: 1,
    value: ["motd=§4§l❀ 乱世方块大陆 ❀ §r", "§b休闲养老 §8· §d趣味玩法 §8· §6交流群:XXX"],
  },
  {
    name: "cyber",
    index: 0,
    value: ["motd=§b§l≋ 赛博方舟 ≋ §9高稳服务器§r", "§7官方网站:www.xxx.com §8丨 §cQQ交流群:XXX"],
  },
  {
    name: "cyber",
    index: 1,
    value: ["motd=§3§l⚡ 未来工艺 ⚡ §b极致优化§r", "§a原版生存§8丨§d副本挑战§8丨§f群聊:XXX"],
  },
  {
    name: "black_gold",
    index: 0,
    value: ["motd=§6§l◆ 天穹领域 ◆ §r", "§8§m————————————§r §7官网:xxx.cn §8QQ群:XXX"],
  },
  {
    name: "black_gold",
    index: 1,
    value: ["motd=§e§l★ 荒古纪元 大型服 ★§r", "§c禁止破坏熊孩子 §8| §5社群:XXX §9长期开放"],
  },
  {
    name: "gradient",
    index: 0,
    value: ["motd=§a§l● §b欢乐方块世界 §d●§r", "§f§m════════════§r §7QQ群:XXX §6欢迎入驻"],
  },
  {
    name: "gradient",
    index: 1,
    value: ["motd=§c§l❖ 缤纷冒险大陆 ❖ §r", "§9建筑丨§b空岛丨§a生存丨§d群号:XXX"],
  },
  {
    name: "minimal",
    index: 0,
    value: ["motd=§7§l▌ 静谧之地 ▌ §f原版纯净体验§r", "§8官网:server-xxx.com §7丨 §aQQ社群:XXX"],
  },
  {
    name: "minimal",
    index: 1,
    value: ["motd=§8§l▪ 永恒云端 ▪ §b稳定低延迟§r", "§6专属福利 §8| §d玩家交流群:XXX"],
  },
  {
    name: "dark",
    index: 0,
    value: ["motd=§4§l☠ 末世荒原 ☠ §c硬核生存§r", "§7资源争夺 §8· §5公会系统 §8· QQ:XXX"],
  },
  {
    name: "dark",
    index: 1,
    value: ["motd=§5§l♛ 暗域王朝 ♛ §r", "§d独特玩法 §9联机畅玩 §8丨§a官方群:XXX"],
  },
  {
    name: "symbols",
    index: 0,
    value: ["motd=§e§l✦✦ 梦幻方块国度 ✦✦§r", "§b福利多多 §7| §6官网链接 §7| §cQQ群:XXX"],
  },
  {
    name: "symbols",
    index: 1,
    value: ["motd=§9§l┏━ 星界冒险 ━┓ §r", "§a原版增强 §8· §e新手礼包 §8· 群:XXX"],
  },
  {
    name: "cozy",
    index: 0,
    value: ["motd=§d§l☁ 慢节奏养老小镇 ☁§r", "§f自由建筑 §7丨 §b和谐社区 §7丨 QQ:XXX"],
  },
  {
    name: "cozy",
    index: 1,
    value: ["motd=§a§l❁ 林间闲世 ❁ §f轻松游玩§r", "§9无暴力肝度 §8| §5玩家互助群:XXX"],
  },
];
