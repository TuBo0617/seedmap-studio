export interface FeatureMeta {
  label: string
  color: string
  short: string
}

export const FEATURE_META: Record<string, FeatureMeta> = {
  village: { label: '村庄', color: '#f2b134', short: '村' },
  outpost: { label: '掠夺者前哨站', color: '#ef5350', short: '哨' },
  shipwreck: { label: '沉船', color: '#26a69a', short: '船' },
  monument: { label: '海底神殿', color: '#4fc3f7', short: '殿' },
  mansion: { label: '林地府邸', color: '#b06cf0', short: '邸' },
  'ancient-city': { label: '远古城市', color: '#5c6bc0', short: '古' },
  'trail-ruins': { label: '古迹废墟', color: '#a1887f', short: '迹' },
  'trial-chambers': { label: '试炼密室', color: '#ffb74d', short: '试' },
  'ruined-portal': { label: '废弃传送门', color: '#ab47bc', short: '门' },
  stronghold: { label: '要塞', color: '#9aa0a6', short: '堡' },
  'slime-chunk': { label: '史莱姆区块', color: '#66bb6a', short: '粘' },
  'desert-pyramid': { label: '沙漠神殿', color: '#ffca28', short: '沙' },
  'jungle-temple': { label: '丛林神庙', color: '#43a047', short: '丛' },
  'swamp-hut': { label: '沼泽小屋', color: '#7e57c2', short: '沼' },
  igloo: { label: '雪屋', color: '#90caf9', short: '雪' },
  'ocean-ruin': { label: '海底废墟', color: '#4dd0e1', short: '墟' },
  'buried-treasure': { label: '埋藏的宝藏', color: '#ffd54f', short: '宝' },
  mineshaft: { label: '废弃矿井', color: '#8d6e63', short: '矿' },
  'desert-well': { label: '沙漠水井', color: '#d4a373', short: '井' },
  geode: { label: '紫晶洞', color: '#ce93d8', short: '晶' },
  fortress: { label: '下界要塞', color: '#e57373', short: '堡' },
  bastion: { label: '堡垒遗迹', color: '#ff8a65', short: '垒' },
  'end-city': { label: '末地城', color: '#b39ddb', short: '城' },
  'end-gateway': { label: '末地折跃门', color: '#9575cd', short: '跃' },
  'abandoned-camp': { label: '废弃营地', color: '#aed581', short: '营' },
  'nether-fossil': { label: '下界化石', color: '#bcaaa4', short: '化' },
}

export const featureMeta = (kind: string): FeatureMeta =>
  FEATURE_META[kind] ?? { label: kind, color: '#c0c0c8', short: '?' }

export const LEGEND_ORDER = Object.keys(FEATURE_META)

/** 与 seedmap-core 的 FEATURE_* 位一一对应。 */
export const FEATURE_BITS: Record<string, number> = {
  village: 1 << 0,
  outpost: 1 << 1,
  shipwreck: 1 << 2,
  monument: 1 << 3,
  mansion: 1 << 4,
  'ancient-city': 1 << 5,
  'trail-ruins': 1 << 6,
  'trial-chambers': 1 << 7,
  'ruined-portal': 1 << 8,
  stronghold: 1 << 9,
  'slime-chunk': 1 << 10,
  'desert-pyramid': 1 << 11,
  'jungle-temple': 1 << 12,
  'swamp-hut': 1 << 13,
  igloo: 1 << 14,
  'ocean-ruin': 1 << 15,
  'buried-treasure': 1 << 16,
  mineshaft: 1 << 17,
  'desert-well': 1 << 18,
  geode: 1 << 19,
  fortress: 1 << 20,
  bastion: 1 << 21,
  'end-city': 1 << 22,
  'end-gateway': 1 << 23,
  'abandoned-camp': 1 << 24,
  'nether-fossil': 1 << 25,
}

/** 默认只打开值得跑过去的结构，避免一屏全是点。 */
export const DEFAULT_KINDS = [
  'village',
  'outpost',
  'monument',
  'mansion',
  'ancient-city',
  'trail-ruins',
  'trial-chambers',
  'stronghold',
  'fortress',
  'bastion',
  'end-city',
]
