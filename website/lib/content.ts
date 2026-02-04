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
    tagline: 'Advanced Computing Research',
    title: 'Rogue Intelligence',
    subtitle: 'LNC',
    description: 'Pioneering the future of adaptive computing through self-hosting programming languages, intelligent operating systems, and exact mathematical frameworks.'
  },
  metrics: [
    { label: 'Core Projects', value: '4' },
    { label: 'Performance Gain', value: '2.8×' },
    { label: 'Self-Hosting', value: '100%' },
    { label: 'Rounding Errors', value: '0' },
  ],
  philosophy: {
    title: 'Our Philosophy',
    items: [
      { title: 'Learn, Don\'t Assume', description: 'Let systems discover optimal configurations through observation.' },
      { title: 'Both, Not Either', description: 'Refuse false dichotomies—combine power with simplicity.' },
      { title: 'Prevent, Don\'t React', description: 'Design systems that fix problems before they cause failures.' },
      { title: 'Adapt, Don\'t Hardcode', description: 'Everything should be dynamic and determined at runtime.' },
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
