function lerp(a, b, t) {
  return [
    Math.round(a[0] + (b[0] - a[0]) * t),
    Math.round(a[1] + (b[1] - a[1]) * t),
    Math.round(a[2] + (b[2] - a[2]) * t)
  ]
}

function lerpRgb(a, b, t) {
  const c = lerp(a, b, t)
  return `rgb(${c[0]},${c[1]},${c[2]})`
}

function modulate(base, connectivity, isLight = false) {
  const factor = isLight
    ? 0.35 + connectivity * 0.65
    : 0.6 + connectivity * 0.4
  return `rgb(${Math.round(base[0] * factor)},${Math.round(base[1] * factor)},${Math.round(base[2] * factor)})`
}

function rgbString(rgb) {
  return `rgb(${rgb[0]},${rgb[1]},${rgb[2]})`
}

function nodeColor(theme, connectivity, clusterIndex) {
  const isLight = theme.isLight || false
  if (clusterIndex !== undefined && theme.palette.length > 0) {
    return modulate(theme.palette[clusterIndex % theme.palette.length], connectivity, isLight)
  }
  const c = lerp(theme.nodeMin, theme.nodeMax, connectivity)
  return `rgb(${c[0]},${c[1]},${c[2]})`
}

function edgeColor(theme, weight) {
  const c = lerp(theme.edgeMin, theme.edgeMax, weight)
  return `rgb(${c[0]},${c[1]},${c[2]})`
}

const CANVAS_THEME_DEFS = [
  {
    id: 'ember',
    name: 'Ember',
    dark: {
      background: '#1a1816',
      nodeMin: [170, 110, 70],
      nodeMax: [230, 50, 40],
      palette: [
        [230, 80, 50],
        [220, 160, 60],
        [180, 60, 90],
        [240, 120, 40],
        [160, 90, 140],
        [200, 140, 80],
        [220, 60, 120],
        [170, 120, 50]
      ],
      edgeMin: [45, 30, 25],
      edgeMax: [160, 60, 40],
      labelColor: 'rgb(200, 175, 155)',
      labelBg: 'rgb(24, 20, 18)',
      labelBorder: 'rgba(140, 100, 70, 0.3)',
      nodeLabelColor: '#b0a090'
    },
    light: {
      background: '#faf6f2',
      nodeMin: [190, 140, 110],
      nodeMax: [200, 60, 30],
      palette: [
        [200, 70, 40],
        [190, 140, 30],
        [160, 50, 70],
        [210, 100, 20],
        [140, 70, 120],
        [180, 120, 50],
        [200, 50, 100],
        [150, 100, 30]
      ],
      edgeMin: [220, 200, 190],
      edgeMax: [180, 80, 40],
      labelColor: 'rgb(90, 55, 30)',
      labelBg: 'rgb(255, 252, 248)',
      labelBorder: 'rgba(180, 120, 80, 0.25)',
      nodeLabelColor: '#6d4c3a'
    }
  },
  {
    id: 'steel-violet',
    name: 'Steel Violet',
    dark: {
      background: '#1a1a1a',
      nodeMin: [100, 115, 175],
      nodeMax: [130, 50, 230],
      palette: [
        [140, 80, 220],
        [80, 140, 210],
        [180, 70, 160],
        [90, 170, 180],
        [200, 100, 120],
        [110, 120, 200],
        [160, 140, 80],
        [100, 180, 130]
      ],
      edgeMin: [30, 30, 45],
      edgeMax: [80, 65, 160],
      labelColor: 'rgb(160, 175, 200)',
      labelBg: 'rgb(22, 22, 22)',
      labelBorder: 'rgba(80, 100, 140, 0.3)',
      nodeLabelColor: '#8899b0'
    },
    light: {
      background: '#f5f5fa',
      nodeMin: [120, 130, 180],
      nodeMax: [110, 40, 200],
      palette: [
        [120, 60, 200],
        [50, 110, 190],
        [150, 50, 140],
        [60, 140, 155],
        [170, 70, 100],
        [80, 90, 180],
        [130, 110, 50],
        [70, 150, 100]
      ],
      edgeMin: [200, 200, 220],
      edgeMax: [90, 60, 170],
      labelColor: 'rgb(50, 45, 80)',
      labelBg: 'rgb(252, 252, 255)',
      labelBorder: 'rgba(90, 80, 150, 0.22)',
      nodeLabelColor: '#505570'
    }
  },
  {
    id: 'aurora',
    name: 'Aurora',
    dark: {
      background: '#141a1a',
      nodeMin: [60, 160, 140],
      nodeMax: [140, 60, 220],
      palette: [
        [60, 190, 160],
        [140, 80, 210],
        [80, 160, 220],
        [200, 90, 140],
        [100, 200, 100],
        [180, 140, 60],
        [70, 130, 200],
        [190, 120, 180]
      ],
      edgeMin: [25, 40, 38],
      edgeMax: [70, 100, 150],
      labelColor: 'rgb(155, 200, 190)',
      labelBg: 'rgb(18, 22, 22)',
      labelBorder: 'rgba(70, 140, 130, 0.3)',
      nodeLabelColor: '#88b0a8'
    },
    light: {
      background: '#f0f8f6',
      nodeMin: [40, 140, 120],
      nodeMax: [120, 50, 190],
      palette: [
        [30, 160, 135],
        [115, 55, 180],
        [50, 130, 190],
        [170, 60, 110],
        [60, 170, 70],
        [150, 110, 30],
        [40, 105, 170],
        [160, 85, 150]
      ],
      edgeMin: [200, 220, 218],
      edgeMax: [60, 90, 140],
      labelColor: 'rgb(30, 70, 65)',
      labelBg: 'rgb(250, 255, 254)',
      labelBorder: 'rgba(50, 120, 110, 0.22)',
      nodeLabelColor: '#3d7a72'
    }
  },
  {
    id: 'midnight',
    name: 'Midnight',
    dark: {
      background: '#12141c',
      nodeMin: [70, 90, 150],
      nodeMax: [100, 140, 255],
      palette: [
        [90, 130, 240],
        [60, 180, 190],
        [150, 90, 220],
        [80, 190, 140],
        [180, 80, 160],
        [100, 160, 210],
        [200, 130, 80],
        [120, 100, 220]
      ],
      edgeMin: [25, 28, 50],
      edgeMax: [55, 80, 170],
      labelColor: 'rgb(150, 170, 210)',
      labelBg: 'rgb(16, 18, 26)',
      labelBorder: 'rgba(70, 90, 150, 0.3)',
      nodeLabelColor: '#7088b0'
    },
    light: {
      background: '#f0f2fa',
      nodeMin: [90, 110, 160],
      nodeMax: [70, 100, 230],
      palette: [
        [70, 100, 210],
        [40, 150, 160],
        [130, 60, 190],
        [50, 160, 110],
        [150, 50, 130],
        [70, 130, 180],
        [170, 100, 50],
        [90, 70, 190]
      ],
      edgeMin: [210, 212, 225],
      edgeMax: [60, 80, 170],
      labelColor: 'rgb(35, 45, 80)',
      labelBg: 'rgb(255, 255, 255)',
      labelBorder: 'rgba(60, 80, 150, 0.18)',
      nodeLabelColor: '#4a5580'
    }
  },
  {
    id: 'monochrome',
    name: 'Mono',
    dark: {
      background: '#181818',
      nodeMin: [100, 100, 100],
      nodeMax: [220, 220, 220],
      palette: [
        [180, 180, 180],
        [140, 140, 140],
        [200, 200, 200],
        [120, 120, 120],
        [160, 160, 160],
        [190, 190, 190],
        [130, 130, 130],
        [170, 170, 170]
      ],
      edgeMin: [35, 35, 35],
      edgeMax: [100, 100, 100],
      labelColor: 'rgb(180, 180, 180)',
      labelBg: 'rgb(20, 20, 20)',
      labelBorder: 'rgba(100, 100, 100, 0.3)',
      nodeLabelColor: '#909090'
    },
    light: {
      background: '#f5f5f5',
      nodeMin: [80, 80, 80],
      nodeMax: [50, 50, 50],
      palette: [
        [60, 60, 60],
        [90, 90, 90],
        [45, 45, 45],
        [100, 100, 100],
        [70, 70, 70],
        [55, 55, 55],
        [85, 85, 85],
        [65, 65, 65]
      ],
      edgeMin: [210, 210, 210],
      edgeMax: [140, 140, 140],
      labelColor: 'rgb(50, 50, 50)',
      labelBg: 'rgb(255, 255, 255)',
      labelBorder: 'rgba(130, 130, 130, 0.22)',
      nodeLabelColor: '#555555'
    }
  }
]

function resolveCanvasTheme(themeDef, mode) {
  const variant = mode === 'light' ? themeDef.light : themeDef.dark
  return { ...variant, id: themeDef.id, name: themeDef.name, isLight: mode === 'light' }
}

function getCanvasTheme(themeId, mode) {
  const def = CANVAS_THEME_DEFS.find((t) => t.id === themeId) || CANVAS_THEME_DEFS.find((t) => t.id === 'midnight')
  return resolveCanvasTheme(def, mode)
}

const CANVAS_THEMES = CANVAS_THEME_DEFS.map((def) => ({ ...def.dark, id: def.id, name: def.name }))

const DEFAULT_THEME = CANVAS_THEMES.find((t) => t.id === 'midnight')

export {
  CANVAS_THEME_DEFS,
  CANVAS_THEMES,
  DEFAULT_THEME,
  resolveCanvasTheme,
  getCanvasTheme,
  nodeColor,
  edgeColor,
  lerpRgb,
  rgbString,
  lerp as lerpRgbTriple
}
