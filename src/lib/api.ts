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
