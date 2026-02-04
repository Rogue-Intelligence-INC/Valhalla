// Content Management System Types and Utilities

export interface HeroContent {
  tagline: string
  title: string
  subtitle: string
  description: string
}

export interface MetricItem {
  label: string
  value: string
}

export interface PhilosophyItem {
  title: string
  description: string
}

export interface SiteContent {
  hero: HeroContent
  metrics: MetricItem[]
  philosophy: {
    title: string
    items: PhilosophyItem[]
  }
  updatedAt?: string
}

export interface VideoItem {
  id: string
  title: string
  description: string
  path: string
  thumbnail?: string
  duration?: string
  category: string
  addedAt: string
}

export interface VideoLibrary {
  videos: VideoItem[]
  categories: string[]
}

// Default content
export const defaultSiteContent: SiteContent = {
  hero: {
    tagline: 'Project Valhalla',
    title: 'Rogue Intelligence',
    subtitle: 'LNC',
    description: 'Where exact mathematics meets physics-inspired computing. Pioneering adaptive intelligence through self-hosting languages, intelligent operating systems, and exact mathematical frameworks.'
  },
  metrics: [
    { label: 'Core Projects', value: '6' },
    { label: 'Performance Gain', value: '2.8×' },
    { label: 'Self-Hosting', value: '100%' },
    { label: 'Rounding Errors', value: '0' },
  ],
  philosophy: {
    title: 'Philosophy',
    items: [
      { 
        title: 'Learn, Don\'t Assume', 
        description: 'Let systems discover optimal configurations through observation rather than predetermined rules.' 
      },
      { 
        title: 'Both, Not Either', 
        description: 'Refuse false dichotomies—combine power with simplicity, exactness with performance.' 
      },
      { 
        title: 'Prevent, Don\'t React', 
        description: 'Design systems that fix problems before they cause failures, not after.' 
      },
      { 
        title: 'Adapt, Don\'t Hardcode', 
        description: 'Everything should be dynamic and determined at runtime based on actual conditions.' 
      },
    ]
  }
}

export const defaultVideoLibrary: VideoLibrary = {
  videos: [],
  categories: ['Research', 'Demos', 'Tutorials', 'Presentations']
}

// Local Storage Keys
const CONTENT_KEY = 'site-content'
const VIDEO_KEY = 'video-library'

// Content Management Functions
export function loadSiteContent(): SiteContent {
  if (typeof window === 'undefined') return defaultSiteContent
  
  try {
    const saved = localStorage.getItem(CONTENT_KEY)
    if (saved) {
      return JSON.parse(saved)
    }
  } catch (e) {
    console.error('Failed to load site content:', e)
  }
  return defaultSiteContent
}

export function saveSiteContent(content: SiteContent): void {
  if (typeof window === 'undefined') return
  
  try {
    content.updatedAt = new Date().toISOString()
    localStorage.setItem(CONTENT_KEY, JSON.stringify(content))
  } catch (e) {
    console.error('Failed to save site content:', e)
  }
}

export function resetSiteContent(): SiteContent {
  if (typeof window === 'undefined') return defaultSiteContent
  
  localStorage.removeItem(CONTENT_KEY)
  return defaultSiteContent
}

// Video Library Functions
export function loadVideoLibrary(): VideoLibrary {
  if (typeof window === 'undefined') return defaultVideoLibrary
  
  try {
    const saved = localStorage.getItem(VIDEO_KEY)
    if (saved) {
      return JSON.parse(saved)
    }
  } catch (e) {
    console.error('Failed to load video library:', e)
  }
  return defaultVideoLibrary
}

export function saveVideoLibrary(library: VideoLibrary): void {
  if (typeof window === 'undefined') return
  
  try {
    localStorage.setItem(VIDEO_KEY, JSON.stringify(library))
  } catch (e) {
    console.error('Failed to save video library:', e)
  }
}

export function addVideo(video: Omit<VideoItem, 'id' | 'addedAt'>): VideoItem {
  const library = loadVideoLibrary()
  const newVideo: VideoItem = {
    ...video,
    id: crypto.randomUUID(),
    addedAt: new Date().toISOString()
  }
  library.videos.unshift(newVideo)
  saveVideoLibrary(library)
  return newVideo
}

export function removeVideo(id: string): void {
  const library = loadVideoLibrary()
  library.videos = library.videos.filter(v => v.id !== id)
  saveVideoLibrary(library)
}

export function addCategory(category: string): void {
  const library = loadVideoLibrary()
  if (!library.categories.includes(category)) {
    library.categories.push(category)
    saveVideoLibrary(library)
  }
}

// Generate video thumbnail from video file
export function generateThumbnail(videoElement: HTMLVideoElement): Promise<string> {
  return new Promise((resolve) => {
    const canvas = document.createElement('canvas')
    canvas.width = videoElement.videoWidth
    canvas.height = videoElement.videoHeight
    const ctx = canvas.getContext('2d')
    if (ctx) {
      ctx.drawImage(videoElement, 0, 0, canvas.width, canvas.height)
      resolve(canvas.toDataURL('image/jpeg', 0.7))
    } else {
      resolve('')
    }
  })
}

// Format duration from seconds
export function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  
  if (h > 0) {
    return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  }
  return `${m}:${s.toString().padStart(2, '0')}`
}

// Project Data
export interface Project {
  id: string
  title: string
  subtitle: string
  description: string
  features: string[]
  status: 'active' | 'research' | 'experimental'
  href: string
}

export const projects: Project[] = [
  {
    id: 'wave',
    title: 'Wave',
    subtitle: 'Self-Hosting Programming Language',
    description: 'Rule-derived compilation through the Unified Field (i, e, r). Self-hosting compiler achieves 2.8× speedup over Python with zero external dependencies.',
    features: ['Self-Hosting', '2.8× Speed', 'Fate Optimization', 'Zero Dependencies'],
    status: 'active',
    href: '/projects/wave',
  },
  {
    id: 's-wind',
    title: 'S-wind',
    subtitle: 'Autonomous Algebraic Discovery',
    description: 'Exhaustive algebraic exploration over ℚ using Extended Precision Rationals (EPR). Mathematical closure certification with self-loop pattern discovery.',
    features: ['EPR Arithmetic', 'Closure Proof', 'LaTeX Output', 'OEIS Integration'],
    status: 'experimental',
    href: '/projects/s-wind',
  },
  {
    id: 'val-os',
    title: 'Val-OS',
    subtitle: 'Wave Language Operating System',
    description: 'Experimental OS kernel written entirely in Wave. Fate+Quad memory management with zero hardcoded rules.',
    features: ['Wave Kernel', 'Fate+Quad Memory', 'Desktop Environment', 'App Store'],
    status: 'research',
    href: '/projects/val-os',
  },
  {
    id: 'val-crypt',
    title: 'Val-Crypt',
    subtitle: 'MeshBrain 2D GPU Encryption',
    description: 'Neural network-inspired encryption with 2D spatial processing. GPU acceleration via wgpu. 42% avalanche effect.',
    features: ['42% Avalanche', 'GPU Accelerated', '2D Spatial', 'CPU Parallel'],
    status: 'research',
    href: '/projects/val-crypt',
  },
  {
    id: 'valhalla-core',
    title: 'Valhalla Core',
    subtitle: 'Unified Algorithm Architecture',
    description: 'Foundational quad-system: BlackHole, MeshBrain, MultiNova, BaseForce. Unified by Fate coordination.',
    features: ['Quad System', 'Fate Coordination', 'Physics-Inspired', 'Cross-Disciplinary'],
    status: 'research',
    href: '/projects/valhalla',
  },
  {
    id: 'valhalla-compression',
    title: 'Valhalla Compression',
    subtitle: 'Intelligent Quad-System Processing',
    description: 'Feature extraction drives Fate routing. Automatic mode switching: learning vs compression.',
    features: ['Fate Routing', 'Feature Extraction', 'Dual Mode', 'Adaptive'],
    status: 'experimental',
    href: '/projects/compression',
  },
]
