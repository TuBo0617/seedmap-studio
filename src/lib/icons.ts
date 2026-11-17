/**
 * 图标资源来自原美西螈启动器的种子地图（public/seed-map-assets）。
 * structures: 80×80 的方块缩略图，用于地图标记与图例；
 * ores: 3D 方块图标用于面板，ore-flat 的 16×16 平铺纹理用于地图点；
 * biomes: 群系缩略图，用于群系选择器。
 */

export const ASSET_ROOT = `${import.meta.env.BASE_URL}seed-map-assets`

const structure = (name: string) => `${ASSET_ROOT}/structures/${name}`
const ore = (name: string) => `${ASSET_ROOT}/ores/${name}`
const oreFlat = (name: string) => `${ASSET_ROOT}/ores/ore-flat/${name}`
const biome = (name: string) => `${ASSET_ROOT}/biomes/${name}.webp`

export const STRUCTURE_ICONS: Record<string, string> = {
  village: structure('village.webp'),
  outpost: structure('pillager_outpost.webp'),
  shipwreck: structure('shipwreck.webp'),
  monument: structure('ocean_monument.webp'),
  mansion: structure('woodland_mansion.webp'),
  'ancient-city': structure('ancient_city.webp'),
  'trail-ruins': structure('trail_ruin.webp'),
  'trial-chambers': structure('trial_chambers.webp'),
  'ruined-portal': structure('ruined_portal.webp'),
  stronghold: structure('stronghold.webp'),
  'slime-chunk': structure('slime_chunks.webp'),
  'desert-pyramid': structure('desert_pyramid.webp'),
  'jungle-temple': structure('jungle_temple.webp'),
  'swamp-hut': structure('swamp_hut.webp'),
  igloo: structure('igloo.webp'),
  'ocean-ruin': structure('ocean_ruin.webp'),
  'buried-treasure': structure('buried_treasure.webp'),
  mineshaft: structure('mineshaft.webp'),
  'desert-well': structure('desert_well.webp'),
  geode: structure('amethyst_geode.webp'),
  fortress: structure('nether_fortress.webp'),
  bastion: structure('bastion_remnant.webp'),
  'end-city': structure('end_city.webp'),
  'end-gateway': structure('end_gateway.webp'),
  'abandoned-camp': structure('abandoned_camp.png'),
  'nether-fossil': structure('nether_fossil.png'),
}

export const END_CITY_SHIP_ICON = structure('end_city_ship.webp')
export const END_CITY_NO_SHIP_ICON = structure('end_city_no_ship.webp')
export const SPAWN_ICON = structure('spawn_point.webp')

export const structureIcon = (kind: string, endShip?: boolean): string | undefined => {
  if (kind === 'end-city' && endShip !== undefined) {
    return endShip ? END_CITY_SHIP_ICON : END_CITY_NO_SHIP_ICON
  }
  return STRUCTURE_ICONS[kind]
}

/** 面板里用的 3D 方块图标；没有素材的矿物返回 undefined，界面回退到色块。 */
export const ORE_ITEM_ICONS: Record<string, string> = {
  diamond: ore('diamond-ore.png'),
  iron: ore('iron-ore.png'),
  iron_vein: ore('block-of-raw-iron.png'),
  copper: ore('copper-ore.png'),
  copper_vein: ore('block-of-raw-copper.png'),
  gold: ore('gold-ore.png'),
  redstone: ore('redstone-ore.png'),
  lapis: ore('lapis-lazuli-ore.png'),
  coal: ore('coal-ore.png'),
  netherite: ore('ancient-debris.png'),
}

/** 地图点用的平铺纹理，深板岩变体在 y < 0 时使用。 */
const ORE_TILE_NAMES: Record<string, [string, string?]> = {
  coal: ['coal_ore.png', 'deepslate_coal_ore.png'],
  copper: ['copper_ore.png', 'deepslate_copper_ore.png'],
  diamond: ['diamond_ore.png', 'deepslate_diamond_ore.png'],
  gold: ['gold_ore.png', 'deepslate_gold_ore.png'],
  iron: ['iron_ore.png', 'deepslate_iron_ore.png'],
  lapis: ['lapis_ore.png', 'deepslate_lapis_ore.png'],
  redstone: ['redstone_ore.png', 'deepslate_redstone_ore.png'],
  netherite: ['ancient_debris_side.png'],
}

export const oreTileIcon = (kind: string, deepslate: boolean): string | undefined => {
  const entry = ORE_TILE_NAMES[kind]
  if (!entry) return undefined
  const name = deepslate && entry[1] ? entry[1] : entry[0]
  return oreFlat(name)
}

export const oreItemIcon = (kind: string): string | undefined => ORE_ITEM_ICONS[kind]

/** 群系 id → 缩略图。只有素材里存在的群系才有图，其余返回 undefined。 */
export const BIOME_IMAGE_SLUGS: Record<number, string> = {
  0: 'ocean',
  1: 'plains',
  2: 'desert',
  3: 'windswept-hills',
  4: 'forest',
  5: 'taiga',
  6: 'swamp',
  7: 'river',
  8: 'nether-wastes',
  9: 'the-end',
  10: 'frozen-ocean',
  11: 'frozen-river',
  12: 'snowy-plains',
  13: 'snowy-slopes',
  14: 'mushroom-fields',
  16: 'beach',
  21: 'jungle',
  24: 'deep-ocean',
  25: 'stony-shore',
  26: 'snowy-beach',
  27: 'birch-forest',
  29: 'dark-forest',
  30: 'snowy-taiga',
  32: 'old-growth-pine-taiga',
  34: 'windswept-gravelly-hills',
  35: 'savanna',
  36: 'savanna-plateau',
  37: 'badlands',
  38: 'wooded-badlands',
  40: 'small-end-islands',
  41: 'end-midlands',
  42: 'end-highlands',
  43: 'end-barrens',
  44: 'warm-ocean',
  45: 'lukewarm-ocean',
  46: 'cold-ocean',
  48: 'deep-lukewarm-ocean',
  49: 'deep-cold-ocean',
  50: 'deep-frozen-ocean',
  129: 'sunflower-plains',
  131: 'windswept-gravelly-hills',
  132: 'flower-forest',
  140: 'ice-spikes',
  149: 'jungle',
  155: 'old-growth-birch-forest',
  160: 'old-growth-spruce-taiga',
  165: 'eroded-badlands',
  168: 'bamboo-jungle',
  170: 'soul-sand-valley',
  171: 'crimson-forest',
  172: 'warped-forest',
  173: 'basalt-deltas',
  174: 'dripstone-caves',
  175: 'lush-caves',
  176: 'meadow',
  177: 'grove',
  178: 'snowy-slopes',
  179: 'snowy-plains',
  180: 'frozen-peaks',
  181: 'jagged-peaks',
  182: 'stony-peaks',
  183: 'deep-dark',
  184: 'mangrove-swamp',
  185: 'cherry-grove',
  186: 'pale-garden',
}

export const biomeImage = (id: number): string | undefined => {
  const slug = BIOME_IMAGE_SLUGS[id]
  return slug ? biome(slug) : undefined
}

// --- image cache ---------------------------------------------------------

const images = new Map<string, HTMLImageElement>()
const pending = new Map<string, Promise<HTMLImageElement>>()
let readyHandler: (() => void) | null = null

/** 主画布注册一个“有新图标就重绘”的回调。 */
export function setIconReadyHandler(handler: (() => void) | null) {
  readyHandler = handler
}

export function cachedImage(url: string | undefined): HTMLImageElement | undefined {
  if (!url) return undefined
  const image = images.get(url)
  return image?.complete && image.naturalWidth > 0 ? image : undefined
}

export function loadImage(url: string): Promise<HTMLImageElement> {
  const done = images.get(url)
  if (done?.complete && done.naturalWidth > 0) return Promise.resolve(done)
  const running = pending.get(url)
  if (running) return running
  const task = new Promise<HTMLImageElement>((resolve, reject) => {
    const image = new Image()
    image.decoding = 'async'
    image.onload = () => {
      images.set(url, image)
      pending.delete(url)
      resolve(image)
      readyHandler?.()
    }
    image.onerror = () => {
      pending.delete(url)
      reject(new Error(`图标加载失败：${url}`))
    }
    image.src = url
  })
  pending.set(url, task)
  return task
}

/** 启动时把所有会用到的图标预热进缓存，之后 canvas 绘制是同步的。 */
export function preloadIcons() {
  const urls = new Set<string>([
    ...Object.values(STRUCTURE_ICONS),
    END_CITY_SHIP_ICON,
    END_CITY_NO_SHIP_ICON,
    SPAWN_ICON,
    ...Object.values(ORE_ITEM_ICONS),
    ...Object.values(BIOME_IMAGE_SLUGS).map((slug) => biome(slug)),
  ])
  for (const [kind, [normal, deep]] of Object.entries(ORE_TILE_NAMES)) {
    void kind
    void loadImage(oreFlat(normal)).catch(() => {})
    if (deep) void loadImage(oreFlat(deep)).catch(() => {})
  }
  for (const url of urls) void loadImage(url).catch(() => {})
}
