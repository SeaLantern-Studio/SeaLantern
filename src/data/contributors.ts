/**
 * 贡献者信息
 *
 * 如果你为 Sea Lantern 做出了贡献，欢迎在这里添加你的信息！
 * 无论是代码、设计、建议、文档还是推广，你的名字都值得被记住。
 */

export type SocialPlatform = "gitee" | "github" | "bilibili" | "qq";

export interface SocialLinks {
  [key: string]: string | undefined;
}

export interface Contributor {
  name: string; // 名字或昵称
  role: string; // 角色描述
  avatar: string; // 头像 URL
  url?: string | SocialLinks; // 可选：个人主页链接或其他链接
}

export const contributors: Contributor[] = [
  {
    name: "FPS_Z",
    role: "创始人 / 主要开发者",
    avatar: "https://api.rms.net.cn/head/FPS_Z",
    url: {
      gitee: "https://gitee.com/fps_z",
      github: "https://github.com/FPSZ",
    },
  },
  {
    name: "烬白Jinby",
    role: "自定义配色/宣传",
    avatar: "https://api.rms.net.cn/head/Jinby_6325",
  },
  {
    name: "NIUNIU3303",
    role: "必火推荐！",
    avatar: "https://api.rms.net.cn/head/NIUNIU3303",
  },
  {
    name: "Little_100",
    role: "打杂",
    avatar: "https://api.rms.net.cn/head/Little100",
    url: {
      gitee: "https://gitee.com/little_100",
      github: "https://github.com/Little100",
      bilibili: "https://space.bilibili.com/1492647738",
      qq: "2662308929",
    },
  },
  {
    name: "MinecraftYJQ",
    role: "小修小改罢",
    avatar: "https://api.rms.net.cn/head/MinecraftYJQ_",
    url: {
      gitee: "https://gitee.com/minecraftyjq",
      github: "https://github.com/MinecraftYJQ",
    },
  },
  {
    name: "HKYZYH",
    role: "修复Wayland协议下白屏问题",
    avatar: "https://api.rms.net.cn/head/HKYZYH",
    url: {
      gitee: "https://gitee.com/HKYZYHgezi",
      github: "https://github.com/HKYZYH",
    },
  },
  {
    name: "清初Lucky",
    role: "喵喵喵~",
    avatar: "https://api.rms.net.cn/head/qingchu2010",
  },
  {
    name: "CmzYa",
    role: "不明所以的commit带来了巨量的体验优化",
    avatar: "https://api.rms.net.cn/head/CmzYa",
    url: {
      github: "https://github.com/CmzYa",
      bilibili: "https://space.bilibili.com/1299848968",
      qq: "2933859893",
      tiktok:
        "https://www.douyin.com/user/MS4wLjABAAAAnaa9DsrNixJyNCFPwtFSTNNI4wrE0ME9nwbiqCrppSIruK_9g-9QRKWbQFPzaPPw",
    },
  },
  {
    name: "LingyeNB",
    role: "+3",
    avatar: "https://api.rms.net.cn/head/LingyeNB",
    url: {
      github: "https://github.com/LingyeNBird",
    },
  },
  {
    name: "皓天是条龙",
    role: "增加了一点新功能",
    avatar: "https://api.rms.net.cn/head/zhu_hao_tian",
    url: {
      github: "https://github.com/zhu1h1t1",
    },
  },
  {
    name: "ieshishinjin",
    role: "新增了功能，并吃了明太鱼干",
    avatar: "https://api.rms.net.cn/head/ieshishinjin",
    url: {
      github: "https://github.com/ieshishinjin",
    },
  },
  {
    name: "欧耶熊猫人",
    role: "Github文档转英文",
    avatar: "https://api.rms.net.cn/head/Pandaman_AF",
    url: {
      github: "https://github.com/PandamanAF",
    },
  },
  {
    name: "猫不笨qwq",
    role: "找到软件图形化错误",
    avatar: "https://api.rms.net.cn/head/maobuben_qwq",
    url: {
      github: "https://github.com/maobuben",
    },
  },
  {
    name: "橙子冰棒",
    role: "修复Java查找算法",
    avatar: "https://api.rms.net.cn/head/TNTNTBTT",
    url: {
      github: "https://github.com/Orange-Icepop",
    },
  },
  {
    name: "xingwangzhe",
    role: "贡献者",
    avatar: "https://api.rms.net.cn/head/xingwangzhe_",
    url: {
      github: "https://github.com/xingwangzhe",
    },
  },
  {
    name: "TNTXZ",
    role: "诶嘿~",
    avatar: "https://api.rms.net.cn/head/_TNTXZ_",
    url: {
      github: "https://github.com/TNTXZ",
    },
  },
  {
    name: "I账户已注销I",
    role: "提出了个性化页面，提供了颜色编辑和颜色选择器",
    avatar: "https://api.rms.net.cn/head/echo500",
    url: {
      github: "https://github.com/zhuxiaojt",
    },
  },
  {
    name: "学渣驹",
    role: "Arch Linux 的 AUR 包维护者",
    avatar: "https://api.rms.net.cn/head/MC_KKY",
    url: {
      github: "https://github.com/xuezhaju",
    },
  },
  {
    name: "NanaLoveyuki",
    role: "欧内该,只要我能帮忙我什么都会做的",
    avatar: "https://api.rms.net.cn/head/NanaLoveyuki",
    url: {
      github: "https://github.com/NanaLoveyuki",
    },
  },

  {
    name: "福瑞控海天",
    role: "海内存知己，天涯若比邻",
    avatar: "https://api.rms.net.cn/head/LucyKitter",
    url: {
      gitee: "https://gitee.com/pnchsb_admin",
    },
  },
  {
    name: "yanuofox",
    role: "可以rua的吉祥物",
    avatar: "https://api.rms.net.cn/head/yanuofox",
    url: {
      github: "https://github.com/foxcyber907",
    },
  },
  {
    name: "Yashiro Nene°",
    role: "幸运☆星",
    avatar: "https://api.rms.net.cn/head/QiuHuang2007",
    url: "https://m.bilibili.com/space/327701",
  },
  {
    name: "NyaCl",
    role: "awa",
    avatar: "https://api.rms.net.cn/head/XueChen_NyaCl",
    url: "",
  },
  {
    name: "龙腾_H",
    role: "贡献者 美术这块 河南卷死我了",
    avatar: "https://api.rms.net.cn/head/Longteng_H",
    url: "https://github.com/longteng-H",
  },
  {
    name: "KercyDing",
    role: "代码审查与CI",
    avatar: "https://api.rms.net.cn/head/KercyDing",
    url: "https://github.com/KercyDing",
  },
  {
    name: "OMIILII",
    role: "精神支柱",
    avatar: "https://api.rms.net.cn/head/Derschnitzelgott",
  },
  {
    name: "Yang458",
    role: "贡献者",
    avatar: "https://api.rms.net.cn/head/Yang4893",
    url: "https://github.com/minecraft-Yang458",
  },
  {
    name: "鸽德迪",
    role: "自定义背景图方案",
    avatar: "https://api.rms.net.cn/head/Alex",
  },
  {
    name: "凋空凌",
    role: "修复文档bug",
    avatar: "https://api.rms.net.cn/head/Alex",
  },
  // ============================================
  // 在这里添加更多贡献者！
  // 没有正版怎么办？
  // 选择皮肤，使用其名字
  // https://www.mcgodx.com/skins/
  // 没有正版怎么办？
  // 选择皮肤，使用其名字
  // https://www.mcgodx.com/skins/
  // ============================================
  // {
  //   name: "你的名字",
  //   role: "贡献者",
  //   avatar: "https://api.rms.net.cn/head/YourName",
  //   url: {
  //     gitee: "https://gitee.com/your-username",
  //     github: "https://github.com/your-username",
  //     bilibili: "https://space.bilibili.com/your-bilibili-id",
  //     qq: "your-qq-number",
  //   },
  // },
];
