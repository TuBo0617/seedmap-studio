import { invoke } from '@tauri-apps/api/core'

export type Edition = 'java' | 'java-large-biomes'
export type Dimension = 'overworld' | 'nether' | 'end'

export interface VersionProfile {
  edition: Edition
  version: string
  available: boolean
  dimensions: Dimension[]
  ores: boolean
  oreKinds: string[]
  featureMasks: number[]
  undergroundMasks: number[]
  undergroundYRanges: number[][]
  biomes: number[]
  engineRevision: string
  note: string | null
}

export interface TileRequest {
  seed: string
  edition: Edition
  version: string
  dimension: Dimension
  x: number
  z: number
  scale: number
  width: number
  height: number
  elevation?: number | null
  terrain: boolean
  contours: boolean
  highlightBiomes?: number[] | null
}

export interface FeatureQuery {
  seed: string
  edition: Edition
  version: string
  dimension: Dimension
  minX: number
  minZ: number
  maxX: number
  maxZ: number
  featureMask?: number | null
}

export interface MapFeature {
  kind: string
  x: number
  z: number
  approximate: boolean
  endShip?: boolean
}

export interface SpawnPoint {
  x: number
  z: number
  approximate: boolean
}

export interface LevelDatInfo {
  seed: string
  version?: string | null
  dataVersion?: number | null
}

export type OreKind =
  | 'diamond'
  | 'iron'
  | 'iron_vein'
  | 'copper'
  | 'copper_vein'
  | 'gold'
  | 'redstone'
  | 'lapis'
  | 'coal'
  | 'netherite'
  | 'emerald'
  | 'quartz'
  | 'nether_gold'
  | 'andesite'
  | 'diorite'
  | 'granite'
  | 'dirt'
  | 'gravel'
  | 'infested'
  | 'deepslate'
  | 'tuff'
  | 'clay'
  | 'blackstone'
  | 'magma'
  | 'soul_sand'

export interface OreHit {
  ore: OreKind
  x: number
  y: number
  z: number
  yMin: number
  yMax: number
  status: string
}

export interface OreScanChunk {
  cx: number
  cz: number
  hits: OreHit[]
  truncated: boolean
}

export interface OreScanRequest {
  seed: string
  edition: Edition
  version: string
  dimension: Dimension
  ores: OreKind[]
  chunks: number[]
}

export interface StructureQuery {
  seed: string
  edition: Edition
  version: string
  dimension: Dimension
  x: number
  z: number
  featureMask: number
  includeBlocks?: boolean
}

export interface StructurePosition {
  x: number
  y: number
  z: number
}

export interface StructurePiece {
  id: number
  name: string
  rotation: number
  position: StructurePosition
  positionKnown: boolean
  min?: StructurePosition
  max?: StructurePosition
  portalEyes?: number
  endPortalRoom?: boolean
  templateId?: string
  mirror?: boolean
}

export interface StructureContainer {
  id: number
  piece: number
  x: number
  z: number
  table?: string | null
  estimated?: boolean
  fixedItems?: { id: string; count: number }[]
}

export interface StructureDetails {
  supported: boolean
  truncated: boolean
  capabilities?: {
    variant?: boolean
    containers?: boolean
    projection?: boolean
    blocks?: boolean
  }
  variant?: {
    rotation: number
    mirror: number
    start: number
    biome: number
    abandoned: boolean
    giant: boolean
    underground: boolean
    airPocket: boolean
    basement: boolean
    cracked: boolean
    size: number
    offset: StructurePosition
    size3d: StructurePosition
  }
  pieces: StructurePiece[]
  containers: StructureContainer[]
  limits: string[]
}

export interface LootItem {
  id: string
  count: number
  enchantments: { id: string; level: number }[]
  effect?: string
  duration?: number
}

export interface StructureLoot {
  supported: boolean
  truncated: boolean
  estimated?: boolean
  items?: LootItem[]
  unresolvedTables?: string[]
  limits: string[]
}

export const fetchProfiles = (): Promise<VersionProfile[]> =>
  invoke<VersionProfile[]>('seedmap_profiles')

export const renderTile = (request: TileRequest): Promise<ArrayBuffer> =>
  invoke<ArrayBuffer>('seedmap_render_tile', { request })

export const findFeatures = (query: FeatureQuery): Promise<MapFeature[]> =>
  invoke<MapFeature[]>('seedmap_find_features', { query })

export const findSpawn = (
  seed: string,
  edition: Edition,
  version: string,
): Promise<SpawnPoint> =>
  invoke<SpawnPoint>('seedmap_spawn', { seed, edition, version })

export const biomeAt = (
  seed: string,
  edition: Edition,
  version: string,
  dimension: Dimension,
  x: number,
  y: number,
  z: number,
): Promise<number> =>
  invoke<number>('seedmap_biome_at', {
    seed,
    edition,
    version,
    dimension,
    x,
    y,
    z,
  })

export const scanOres = (
  request: OreScanRequest,
  requestId: number | null,
): Promise<OreScanChunk[]> =>
  invoke<OreScanChunk[]>('seedmap_scan_ores', { request, requestId })

export const cancelOreScan = (requestId: number): Promise<void> =>
  invoke<void>('seedmap_cancel_ore_scan', { requestId })

export const readLevelDat = (path: string): Promise<LevelDatInfo> =>
  invoke<LevelDatInfo>('seedmap_read_level_dat', { path })

export const structureDetails = (
  query: StructureQuery,
): Promise<StructureDetails> =>
  invoke<StructureDetails>('seedmap_structure_details', { query })

export const containerLoot = (
  query: StructureQuery,
  container: number,
): Promise<StructureLoot> =>
  invoke<StructureLoot>('seedmap_container_loot', { query, container })

export const undergroundTile = (request: {
  tile: TileRequest
  layers: number
}): Promise<ArrayBuffer> =>
  invoke<ArrayBuffer>('seedmap_underground_tile', { request })

export const cancelUnderground = (id: number): Promise<void> =>
  invoke<void>('seedmap_cancel_underground', { id })
