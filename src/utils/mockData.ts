/**
 * 开发者模式 mock 数据
 * dev mode 开启时,serverApi 透明返回这些数据,让侧栏/控制台/选择器等界面均可直接测试
 */

import type { ServerInstance } from "@type/server";
import type { ServerStatusInfo } from "@api/server";

// mock 开关,由 settingsStore 加载后通过 setMockMode 设置
let mockEnabled = false;

export function isMockMode(): boolean {
  return mockEnabled;
}

export function setMockMode(enabled: boolean): void {
  mockEnabled = enabled;
}

// 三个 mock 服务器,覆盖不同核心/版本/状态
const MOCK_SERVERS: ServerInstance[] = [
  {
    id: "mock-survival",
    name: "生存服",
    core_type: "paper",
    core_version: "1.20.4",
    mc_version: "1.20.4",
    path: "/mock/servers/survival",
    jar_path: "/mock/servers/survival/paper-1.20.4.jar",
    startup_mode: "jar",
    java_path: "/usr/bin/java",
    max_memory: 4096,
    min_memory: 1024,
    jvm_args: ["-Xmx4G", "-Xms1G"],
    port: 25565,
    created_at: Math.floor(Date.now() / 1000) - 86400 * 30,
    last_started_at: Math.floor(Date.now() / 1000) - 3600,
  },
  {
    id: "mock-creative",
    name: "创造服",
    core_type: "spigot",
    core_version: "1.19.2",
    mc_version: "1.19.2",
    path: "/mock/servers/creative",
    jar_path: "/mock/servers/creative/spigot-1.19.2.jar",
    startup_mode: "jar",
    java_path: "/usr/bin/java",
    max_memory: 2048,
    min_memory: 512,
    jvm_args: ["-Xmx2G", "-Xms512M"],
    port: 25566,
    created_at: Math.floor(Date.now() / 1000) - 86400 * 15,
    last_started_at: null,
  },
  {
    id: "mock-test",
    name: "测试服",
    core_type: "forge",
    core_version: "40.2.0",
    mc_version: "1.18.2",
    path: "/mock/servers/test",
    jar_path: "/mock/servers/test/forge-1.18.2.jar",
    startup_mode: "jar",
    java_path: "/usr/bin/java",
    max_memory: 6144,
    min_memory: 2048,
    jvm_args: ["-Xmx6G", "-Xms2G"],
    port: 25567,
    created_at: Math.floor(Date.now() / 1000) - 86400 * 7,
    last_started_at: Math.floor(Date.now() / 1000) - 1800,
  },
];

// mock 状态,可被 start/stop 修改
const mockStatuses: Record<string, ServerStatusInfo> = {
  "mock-survival": {
    id: "mock-survival",
    status: "Running",
    pid: 12345,
    uptime: 3600,
  },
  "mock-creative": {
    id: "mock-creative",
    status: "Stopped",
    pid: null,
    uptime: null,
  },
  "mock-test": {
    id: "mock-test",
    status: "Error",
    pid: null,
    uptime: null,
  },
};

// 生存服日志:Paper 完整启动序列 + 插件异常 + 数据库断连 + 玩家超时 + 异步任务崩溃
const SURVIVAL_LOGS: string[] = [
  "[12:00:01] [Server thread/INFO]: Starting minecraft server version 1.20.4",
  "[12:00:01] [Server thread/INFO]: Loading properties",
  "[12:00:01] [Server thread/INFO]: Default game type: SURVIVAL",
  "[12:00:01] [Server thread/INFO]: Generating keypair",
  "[12:00:02] [Server thread/INFO]: Starting Minecraft server on *:25565",
  "[12:00:02] [Server thread/INFO]: Using default channel type",
  "[12:00:02] [Server thread/INFO]: This server is running Paper version git-Paper-448 (MC: 1.20.4) (Implementing API version 1.20.4-R0.1-SNAPSHOT) (Git: 1e3c4f5)",
  '[12:00:02] [Server thread/INFO]: Preparing level "world"',
  "[12:00:03] [Server thread/INFO]: Preparing start region for dimension minecraft:overworld",
  "[12:00:03] [Server thread/INFO]: Time elapsed: 1234 ms",
  '[12:00:03] [Server thread/INFO]: Done (2.456s)! For help, type "help"',
  "[12:00:05] [Server thread/INFO]: [Essentials] Enabling Essentials v2.20.0",
  "[12:00:05] [Server thread/INFO]: [Essentials] Loaded 5 homes, 2 warps",
  "[12:00:06] [Server thread/INFO]: [LuckPerms] Successfully enabled. (author: lucko)",
  "[12:00:08] [Server thread/INFO]: [Vault] Loaded Economy plugin: Essentials Economy",
  "[12:00:12] [Server thread/INFO]: UUID of player Steve is 069a79f4-44e9-4726-a5be-fca90e38aaf5",
  "[12:00:12] [Server thread/INFO]: Steve joined the game",
  "[12:00:15] [Async Chat Thread - #0/INFO]: <Steve> hello everyone",
  "[12:00:20] [Server thread/INFO]: UUID of player Alex is 1c3b6b8f-5c4e-4e1b-9e7e-1d6a4f1dcb5e",
  "[12:00:20] [Server thread/INFO]: Alex joined the game",
  "[12:00:22] [Async Chat Thread - #0/INFO]: <Alex> hi Steve",
  "[12:00:25] [Server thread/WARN]: Can't keep up! Is the server overloaded? Running 2345ms or 47 ticks behind",
  "[12:00:30] [Server thread/WARN]: Steve moved too quickly! from -1.0,64.0,-1.0 to 150.5,64.0,150.5 (150.5 blocks)",
  "[12:00:35] [Server thread/ERROR]: Could not pass event PlayerMoveEvent to Essentials v2.20.0",
  '[12:00:35] [Server thread/ERROR]: java.lang.NullPointerException: Cannot invoke "net.ess3.api.IUser.getHome(String)" because "user" is null',
  "[12:00:35] [Server thread/ERROR]:     at net.ess3.api.IUser.getHome(IUser.java:156)",
  "[12:00:35] [Server thread/ERROR]:     at com.earth2me.essentials.Essentials.onPlayerMove(Essentials.java:342)",
  "[12:00:35] [Server thread/ERROR]:     at com.destroystokyo.paper.event.executor.asm.generated.PaperEventExecutor456.invoke(Unknown Source)",
  "[12:00:35] [Server thread/ERROR]:     at co.aikar.timings.TimedEventExecutor.execute(TimedEventExecutor.java:80)",
  "[12:00:40] [Server thread/WARN]: [LuckPerms] Database connection lost, attempting reconnect...",
  "[12:00:42] [Server thread/ERROR]: [LuckPerms] Failed to reconnect to database: Connection refused",
  "[12:00:42] [Server thread/ERROR]: java.sql.SQLException: Connection is closed",
  "[12:00:42] [Server thread/ERROR]:     at com.zaxxer.hikari.pool.HikariPool.getConnection(HikariPool.java:156)",
  "[12:00:45] [Server thread/INFO]: Steve lost connection: Disconnected",
  "[12:00:45] [Server thread/INFO]: Steve left the game",
  "[12:00:50] [Server thread/WARN]: Alex moved wrongly! from 100.0,64.0,100.0 to 100.5,320.0,100.5",
  "[12:00:55] [Async Chat Thread - #1/INFO]: <Alex> brb",
  "[12:00:58] [Server thread/INFO]: Alex lost connection: Timed out",
  "[12:00:58] [Server thread/INFO]: Alex left the game",
  "[12:01:00] [Server thread/INFO]: [Essentials] Saving homes...",
  "[12:01:00] [Server thread/INFO]: Saved the game",
  "[12:01:05] [Server thread/DEBUG]: Saved chunk (0,0) to disk",
  "[12:01:05] [Server thread/DEBUG]: Saved chunk (-1,0) to disk",
  "[12:01:10] [Server thread/ERROR]: [Async] An exception was thrown by AsyncTask",
  "[12:01:10] [Server thread/ERROR]: java.util.concurrent.RejectedExecutionException: Task rejected due to pool shutdown",
  "[12:01:10] [Server thread/ERROR]:     at java.util.concurrent.ThreadPoolExecutor.reject(ThreadPoolExecutor.java:831)",
  "[12:01:10] [Server thread/ERROR]:     at java.util.concurrent.ThreadPoolExecutor.execute(ThreadPoolExecutor.java:1382)",
  "[12:01:15] [Server thread/INFO]: [Vault] Economy plugin Essentials Economy hooked",
  "[12:01:20] [Server thread/INFO]: [AutoSave] Auto-saved world data",
  "[12:01:25] [Server thread/INFO]: Server thread is running smoothly",
];

// 创造服日志:Spigot 启动 + ANSI 彩色玩家名 + GC 内存警告 + OOM 崩溃停止
const CREATIVE_LOGS: string[] = [
  "[09:00:00] [Server thread/INFO]: Starting minecraft server version 1.19.2",
  "[09:00:01] [Server thread/INFO]: Loading properties",
  "[09:00:01] [Server thread/INFO]: Default game type: CREATIVE",
  "[09:00:01] [Server thread/INFO]: This server is running Spigot version git-Spigot-1.19.2-R0.1-SNAPSHOT",
  '[09:00:02] [Server thread/INFO]: Preparing level "world"',
  "[09:00:03] [Server thread/INFO]: Preparing spawn area: 0%",
  "[09:00:03] [Server thread/INFO]: Preparing spawn area: 25%",
  "[09:00:03] [Server thread/INFO]: Preparing spawn area: 50%",
  "[09:00:04] [Server thread/INFO]: Preparing spawn area: 100%",
  '[09:00:04] [Server thread/INFO]: Done (1.056s)! For help, type "help"',
  "\u001b[0;1m[09:05:00] [Server thread/INFO]: \u001b[36mNotch\u001b[0m joined the game",
  "[09:05:05] [Server thread/INFO]: Notch issued server command: /gamemode creative",
  "[09:05:05] [Server thread/INFO]: Set Notch's game mode to Creative Mode",
  "[09:10:00] [Server thread/WARN]: Notch moved too quickly! from 0.0,64.0,0.0 to 500.0,64.0,500.0",
  "[09:15:00] [Server thread/INFO]: Notch issued server command: /give @s diamond 64",
  "[09:15:00] [Server thread/INFO]: Given 64 Diamond to Notch",
  "[09:20:00] [Server thread/WARN]: GC triggered, freeing 512MB in 45ms",
  "[09:25:00] [Server thread/WARN]: Memory usage high: 85% (3413MB/4096MB)",
  "[09:25:30] [Server thread/WARN]: Can't keep up! Running 5600ms or 112 ticks behind",
  "[09:28:00] [Server thread/WARN]: Memory usage critical: 95% (3891MB/4096MB)",
  "[09:29:00] [Server thread/ERROR]: Failed to allocate chunk data",
  "[09:30:00] [Server thread/ERROR]: Server out of memory",
  "[09:30:00] [Server thread/ERROR]: java.lang.OutOfMemoryError: Java heap space",
  "[09:30:00] [Server thread/ERROR]:     at net.minecraft.world.level.chunk.LevelChunk.<init>(LevelChunk.java:120)",
  "[09:30:00] [Server thread/ERROR]:     at net.minecraft.server.level.ChunkMap.lambda$new$3(ChunkMap.java:85)",
  "[09:30:00] [Server thread/ERROR]:     at java.util.HashMap.resize(HashMap.java:710)",
  "[09:30:00] [Server thread/ERROR]:     at java.util.HashMap.put(HashMap.java:611)",
  "[09:30:01] [Server thread/FATAL]: Server crashed: out of memory",
  "[09:30:01] [Server thread/INFO]: Notch lost connection: Server closed",
  "[09:30:01] [Server thread/INFO]: Notch left the game",
  "[09:30:10] [Server thread/INFO]: Stopping the server",
  "[Sea Lantern] 服务已因内存不足崩溃停止",
];

// 测试服日志:Forge mod 加载链 + 依赖冲突 + Mixin 异常 + ASM 失败 + 完整崩溃堆栈(Caused by 链)
const TEST_LOGS: string[] = [
  "[11:00:00] [main/INFO]: Starting Minecraft 1.18.2 with Forge 40.2.0",
  "[11:00:01] [main/INFO]: Loading Forge 40.2.0",
  "[11:00:02] [main/INFO]: Loading mods",
  "[11:00:02] [modloading-worker-0/INFO]: Loading mod create 0.5.1",
  "[11:00:02] [modloading-worker-1/INFO]: Loading mod jei 9.7.1",
  "[11:00:02] [modloading-worker-2/INFO]: Loading mod flywheel 0.6.4",
  "[11:00:03] [modloading-worker-0/INFO]: create: Initializing Create",
  "[11:00:03] [modloading-worker-1/INFO]: jei: Initializing JEI",
  "[11:00:04] [modloading-worker-2/ERROR]: flywheel: Failed to initialize",
  "[11:00:04] [modloading-worker-2/ERROR]: java.lang.NoClassDefFoundError: com/jozufozu/flywheel/api/FlywheelAPI",
  "[11:00:04] [modloading-worker-2/ERROR]:     at dev.engine_create.Create.onInitialize(Create.java:65)",
  "[11:00:04] [modloading-worker-2/ERROR]:     at net.minecraftforge.fml.javafmlmod.FMLJavaModLoadingContext.lambda$null$0(FMLJavaModLoadingContext.java:67)",
  "[11:00:04] [modloading-worker-2/ERROR]:     at net.minecraftforge.eventbus.EventBus.lambda$post$1(EventBus.java:140)",
  "[11:00:05] [modloading-worker-0/ERROR]: create: Dependency resolution failed",
  "[11:00:05] [modloading-worker-0/ERROR]: create requires flywheel >=0.6.5 but found 0.6.4",
  "[11:00:05] [modloading-worker-0/ERROR]: create requires minecraft >=1.18.2 but found 1.18.1",
  "[11:00:06] [main/ERROR]: Mod dependency resolution failed",
  "[11:00:06] [main/ERROR]: The following mods have dependency issues:",
  "[11:00:06] [main/ERROR]:     - create requires flywheel >=0.6.5 (found 0.6.4)",
  "[11:00:06] [main/ERROR]:     - create requires minecraft >=1.18.2 (found 1.18.1)",
  "[11:00:07] [main/ERROR]: Mixin apply failed: create.mixins.json:LevelChunkMixin from mod create",
  "[11:00:07] [main/ERROR]: org.spongepowered.asm.mixin.transformer.throwables.MixinTransformerError: An unexpected critical error was encountered",
  "[11:00:07] [main/ERROR]:     at org.spongepowered.asm.mixin.transformer.MixinProcessor.applyMixins(MixinProcessor.java:392)",
  "[11:00:07] [main/ERROR]:     at org.spongepowered.asm.mixin.transformer.MixinTransformer.transformClass(MixinTransformer.java:223)",
  "[11:00:07] [main/ERROR]: Caused by: java.lang.RuntimeException: Cannot invoke method create$setBlockEntity on target LevelChunk",
  "[11:00:07] [main/ERROR]:     at dev.engine_create.mixin.LevelChunkMixin.redirect$zzz000(LevelChunkMixin.java:45)",
  "[11:00:07] [main/ERROR]:     at net.minecraft.world.level.chunk.LevelChunk.setBlockState(LevelChunkMixin.java:312)",
  "[11:00:08] [main/ERROR]: ASM patching failed: net.minecraft.world.level.chunk.LevelChunk",
  "[11:00:08] [main/ERROR]: java.lang.RuntimeException: ASM patching failed",
  "[11:00:08] [main/ERROR]:     at net.minecraftforge.coremod.CoreModEngine.transform(CoreModEngine.java:123)",
  "[11:00:08] [main/ERROR]:     at net.minecraftforge.fml.loading.LoadingModList.lambda$dispatchParallelEvent$0(LoadingModList.java:115)",
  "[11:00:08] [main/ERROR]: Caused by: org.objectweb.asm.tree.analysis.AnalyzerException: Error at instruction 45: Expected I but found J",
  "[11:00:08] [main/ERROR]:     at org.objectweb.asm.tree.analysis.Analyzer.analyze(Analyzer.java:301)",
  "[11:00:09] [Server thread/FATAL]: Failed to start the minecraft server",
  "[11:00:09] [Server thread/FATAL]: net.minecraftforge.fml.LoadingFailedException: Loading errors encountered:",
  "[11:00:09] [Server thread/FATAL]:     - create (0.5.1) has failed to load correctly",
  "[11:00:09] [Server thread/FATAL]:     - flywheel (0.6.4) is incompatible with create (0.5.1)",
  "[11:00:09] [Server thread/FATAL]:     - Mixin apply failed: create.mixins.json:LevelChunkMixin",
  "[11:00:09] [Server thread/FATAL]:     - ASM patching failed: net.minecraft.world.level.chunk.LevelChunk",
  "[11:00:10] [main/INFO]: Press any key to continue . . .",
  "[Sea Lantern] 服务器启动失败,请检查 mod 兼容性与依赖关系",
];

const MOCK_LOGS: Record<string, string[]> = {
  "mock-survival": SURVIVAL_LOGS,
  "mock-creative": CREATIVE_LOGS,
  "mock-test": TEST_LOGS,
};

export function getMockServers(): ServerInstance[] {
  return MOCK_SERVERS;
}

export function getMockStatus(id: string): ServerStatusInfo | undefined {
  return mockStatuses[id];
}

export function getMockLogs(id: string, since: number, maxLines?: number): string[] {
  const all = MOCK_LOGS[id] || [];
  // since=0 返回完整历史;since>0 视为已读到最新,返回空
  if (since > 0) return [];
  if (maxLines && maxLines > 0) {
    return all.slice(-maxLines);
  }
  return all;
}

// mock 启动:将状态改为 Starting -> Running
export function mockStart(id: string): void {
  if (!mockStatuses[id]) return;
  mockStatuses[id] = {
    ...mockStatuses[id],
    status: "Starting",
    pid: Math.floor(Math.random() * 100000) + 1000,
    uptime: 0,
  };
  // 2 秒后切到 Running,模拟启动完成
  setTimeout(() => {
    if (mockStatuses[id]) {
      mockStatuses[id] = {
        ...mockStatuses[id],
        status: "Running",
        uptime: 0,
      };
    }
  }, 2000);
}

// mock 停止:直接切到 Stopped
export function mockStop(id: string): void {
  if (!mockStatuses[id]) return;
  mockStatuses[id] = {
    ...mockStatuses[id],
    status: "Stopping",
  };
  setTimeout(() => {
    if (mockStatuses[id]) {
      mockStatuses[id] = {
        ...mockStatuses[id],
        status: "Stopped",
        pid: null,
        uptime: null,
      };
    }
  }, 1000);
}
