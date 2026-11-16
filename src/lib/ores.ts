import type { Dimension, OreKind } from './api'

export interface OreMeta {
  kind: OreKind
  label: string
  color: string
  dimension: Exclude<Dimension, 'end'>
  yMin: number
  yMax: number
  /** 默认打开的矿石：真正值得挖的那几种。 */
  primary: boolean
}

/**
 * 与 seedmap-core 里 `ores::OreKind` 的 snake_case 名字一一对应。
 * 高度区间来自原启动器同一份配置，仅用于界面提示，真正的判定在引擎里。
 */
export const ORES: readonly OreMeta[] = [
  { kind: 'diamond', label: '钻石', color: '#4dd0e1', dimension: 'overworld', yMin: -144, yMax: 16, primary: true },
  { kind: 'iron', label: '铁', color: '#d8c8b4', dimension: 'overworld', yMin: -64, yMax: 319, primary: true },
  { kind: 'iron_vein', label: '铁簇', color: '#ffd8a8', dimension: 'overworld', yMin: -60, yMax: -8, primary: false },
  { kind: 'copper', label: '铜', color: '#e8834a', dimension: 'overworld', yMin: -16, yMax: 112, primary: false },
  { kind: 'copper_vein', label: '铜簇', color: '#ff9f6b', dimension: 'overworld', yMin: 0, yMax: 50, primary: false },
  { kind: 'gold', label: '金', color: '#ffd54f', dimension: 'overworld', yMin: -64, yMax: 32, primary: true },
  { kind: 'redstone', label: '红石', color: '#ef5350', dimension: 'overworld', yMin: -96, yMax: 15, primary: false },
  { kind: 'lapis', label: '青金石', color: '#5c6bc0', dimension: 'overworld', yMin: -64, yMax: 64, primary: false },
  { kind: 'coal', label: '煤', color: '#9e9e9e', dimension: 'overworld', yMin: 0, yMax: 319, primary: true },
  { kind: 'emerald', label: '绿宝石', color: '#66bb6a', dimension: 'overworld', yMin: -64, yMax: 319, primary: false },
  { kind: 'netherite', label: '远古残骸', color: '#8d6e63', dimension: 'nether', yMin: 8, yMax: 119, primary: true },
  { kind: 'quartz', label: '下界石英', color: '#f5f5f5', dimension: 'nether', yMin: 0, yMax: 127, primary: false },
  { kind: 'nether_gold', label: '下界金', color: '#ffb300', dimension: 'nether', yMin: 0, yMax: 127, primary: false },
  { kind: 'andesite', label: '安山岩', color: '#b0bec5', dimension: 'overworld', yMin: -64, yMax: 319, primary: false },
  { kind: 'diorite', label: '闪长岩', color: '#e0e0e0', dimension: 'overworld', yMin: -64, yMax: 319, primary: false },
  { kind: 'granite', label: '花岗岩', color: '#bf8f7a', dimension: 'overworld', yMin: -64, yMax: 319, primary: false },
  { kind: 'dirt', label: '泥土', color: '#8d6e63', dimension: 'overworld', yMin: -64, yMax: 319, primary: false },
  { kind: 'gravel', label: '砂砾', color: '#9e9e9e', dimension: 'overworld', yMin: -64, yMax: 319, primary: false },
  { kind: 'infested', label: '虫蚀石', color: '#7cb342', dimension: 'overworld', yMin: -64, yMax: 319, primary: false },
  { kind: 'deepslate', label: '深板岩', color: '#546e7a', dimension: 'overworld', yMin: -64, yMax: 319, primary: false },
  { kind: 'tuff', label: '凝灰岩', color: '#78909c', dimension: 'overworld', yMin: -64, yMax: 319, primary: false },
  { kind: 'clay', label: '黏土', color: '#90a4ae', dimension: 'overworld', yMin: -64, yMax: 319, primary: false },
  { kind: 'blackstone', label: '黑石', color: '#4a4a52', dimension: 'nether', yMin: 0, yMax: 127, primary: false },
  { kind: 'magma', label: '岩浆块', color: '#ff7043', dimension: 'nether', yMin: 0, yMax: 127, primary: false },
  { kind: 'soul_sand', label: '灵魂沙', color: '#a1887f', dimension: 'nether', yMin: 0, yMax: 127, primary: false },
]

const META = new Map(ORES.map((ore) => [ore.kind, ore]))

export const oreMeta = (kind: string): OreMeta =>
  META.get(kind as OreKind) ?? {
    kind: kind as OreKind,
    label: kind,
    color: '#c0c0c8',
    dimension: 'overworld',
    yMin: 0,
    yMax: 0,
    primary: false,
  }

export const oresForDimension = (dimension: Dimension): OreMeta[] =>
  dimension === 'end' ? [] : ORES.filter((ore) => ore.dimension === dimension)

export const defaultOres = (dimension: Dimension): Set<OreKind> =>
  new Set(oresForDimension(dimension).filter((ore) => ore.primary).map((ore) => ore.kind))
