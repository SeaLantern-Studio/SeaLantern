/**
 * 世界地图接口
 *
 * 四个能力在桌面端与 Web 端行为一致，差异封装在传输层：
 * - 图片：`render_world_map` ↔ `GET /api/instances/{id}/map`，返回 Blob
 * - 摘要：`get_world_map_info` ↔ `GET /api/instances/{id}/map/info`，返回 JSON
 * - 清单：`list_world_map_regions` ↔ `GET /api/instances/{id}/map/regions`，返回 JSON
 * - 玩家：`get_map_player_positions` ↔ `GET /api/instances/{id}/map/players`，返回 JSON
 *
 * 一个 region 覆盖 512×512 方块，坐标对应世界目录下 `r.<x>.<z>.mca` 的 x/z。
 * 渲染结果由后端按 `.mca` 修改时间缓存，重复请求同一块不会重新渲染。
 */

import { invoke, invokeBlob } from "@api/invoke";

/** 维度。三界的 region 目录与坐标空间互相独立。 */
export type MapDimension = "overworld" | "nether" | "end";

/** 全部维度，顺序与游戏内习惯一致。 */
export const MAP_DIMENSIONS: MapDimension[] = ["overworld", "nether", "end"];

/** 地图请求目标：region 坐标缺省 0，维度缺省主世界。 */
export interface WorldMapCoords {
  region_x?: number;
  region_z?: number;
  dimension?: MapDimension;
}

/** 一个已存在的 region 文件概要 */
export interface WorldMapRegion {
  region_x: number;
  region_z: number;
  bytes: number;
  modified_at_unix_secs: number;
}

/** 世界里所有可渲染的 region，`regions` 已按 (x, z) 升序 */
export interface WorldMapRegionList {
  instance_id: string;
  level_name: string;
  dimension: MapDimension;
  regions: WorldMapRegion[];
}

/** 一次渲染的结果摘要 */
export interface WorldMapInfo {
  instance_id: string;
  level_name: string;
  dimension: MapDimension;
  region_x: number;
  region_z: number;
  /** 输出图片边长，恒为 512 */
  pixels: number;
  /** 已渲染的 chunk 数（1.18+ 且已生成） */
  rendered_chunks: number;
  /** 因是 1.12 及更早格式而跳过的 chunk 数，对应像素透明 */
  unsupported_chunks: number;
  /** 因未完全生成而跳过的 chunk 数 */
  unfinished_chunks: number;
  /** 已渲染 chunk 占整块 region 的比例，0~1 */
  coverage: number;
  /** 本次是否命中后端磁盘缓存 */
  from_cache: boolean;
  generated_at_unix_secs: number;
}

/** 一名在线玩家的实时位置（单位：方块） */
export interface PlayerPosition {
  name: string;
  /** 东为正 */
  x: number;
  /** 上为正 */
  y: number;
  /** 南为正 */
  z: number;
  /**
   * 水平朝向（度）。游戏内约定 0 指向正南（+Z），90 指向正西（-X），
   * 画箭头时不能当成数学极角直接用。
   */
  yaw: number;
  dimension: MapDimension;
}

/** 某维度在线玩家的位置快照 */
export interface PlayerPositions {
  instance_id: string;
  dimension: MapDimension;
  captured_at_unix_secs: number;
  players: PlayerPosition[];
}

/** 把目标补成后端期望的完整参数 */
function targetArgs(target: WorldMapCoords): {
  region_x: number;
  region_z: number;
  dimension: MapDimension;
} {
  return {
    region_x: target.region_x ?? 0,
    region_z: target.region_z ?? 0,
    dimension: target.dimension ?? "overworld",
  };
}

export const mapApi = {
  /** 列出实例某个维度里所有可渲染的 region；只扫目录，不触发渲染 */
  listRegions(
    instanceId: string,
    dimension: MapDimension = "overworld",
  ): Promise<WorldMapRegionList> {
    return invoke<WorldMapRegionList>("list_world_map_regions", {
      instance_id: instanceId,
      dimension,
    });
  },

  /** 取渲染摘要；缓存未命中时会顺带生成图片 */
  getInfo(instanceId: string, target: WorldMapCoords = {}): Promise<WorldMapInfo> {
    return invoke<WorldMapInfo>("get_world_map_info", {
      instance_id: instanceId,
      ...targetArgs(target),
    });
  },

  /**
   * 渲染并返回 PNG 的 Blob
   *
   * 调用方负责 `URL.createObjectURL` 并适时 `revokeObjectURL`。
   * 同一 region 的并发请求由后端串行化，重复调用是安全的。
   */
  loadImage(instanceId: string, target: WorldMapCoords = {}): Promise<Blob> {
    return invokeBlob(
      "render_world_map",
      { instance_id: instanceId, ...targetArgs(target) },
      { mimeType: "image/png" },
    );
  },

  /**
   * 查询某维度在线玩家的实时坐标。
   *
   * 语义上区分「没人」和「问不到」：没人在线返回空数组，服务器没开、
   * 没启用 RCON、密码不对会抛错，调用方据此决定是否展示错误提示。
   */
  loadPlayerPositions(
    instanceId: string,
    dimension: MapDimension = "overworld",
  ): Promise<PlayerPositions> {
    return invoke<PlayerPositions>("get_map_player_positions", {
      instance_id: instanceId,
      dimension,
    });
  },
};
