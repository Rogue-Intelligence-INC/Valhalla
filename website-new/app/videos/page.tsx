'use client'

import { useState, useEffect, useRef } from 'react'
import Header from '@/components/Header'
import Footer from '@/components/Footer'
import VideoPlayer from '@/components/VideoPlayer'
import { loadVideoLibrary, VideoItem, VideoLibrary, formatDuration } from '@/lib/content'

export default function VideosPage() {
  const [library, setLibrary] = useState<VideoLibrary>({ videos: [], categories: [] })
  const [selectedVideo, setSelectedVideo] = useState<VideoItem | null>(null)
  const [selectedCategory, setSelectedCategory] = useState<string>('all')
  const [searchQuery, setSearchQuery] = useState('')
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid')
  const [localPath, setLocalPath] = useState('')
  const [scanning, setScanning] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    setLibrary(loadVideoLibrary())
  }, [])

  const filteredVideos = library.videos.filter(video => {
    const matchesCategory = selectedCategory === 'all' || video.category === selectedCategory
    const matchesSearch = video.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
                          video.description.toLowerCase().includes(searchQuery.toLowerCase())
    return matchesCategory && matchesSearch
  })

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files
    if (!files) return

    // Process selected video files
    const newVideos: VideoItem[] = []
    
    Array.from(files).forEach(file => {
      if (file.type.startsWith('video/')) {
        const video: VideoItem = {
          id: crypto.randomUUID(),
          title: file.name.replace(/\.[^/.]+$/, ''),
          description: '',
          path: URL.createObjectURL(file),
          category: 'Uncategorized',
          addedAt: new Date().toISOString()
        }
        newVideos.push(video)
      }
    })

    if (newVideos.length > 0) {
      const updated = {
        ...library,
        videos: [...newVideos, ...library.videos]
      }
      setLibrary(updated)
      localStorage.setItem('video-library', JSON.stringify(updated))
    }
  }

  const handleDrop = (event: React.DragEvent) => {
    event.preventDefault()
    const files = event.dataTransfer.files
    
    const newVideos: VideoItem[] = []
    
    Array.from(files).forEach(file => {
      if (file.type.startsWith('video/')) {
        const video: VideoItem = {
          id: crypto.randomUUID(),
          title: file.name.replace(/\.[^/.]+$/, ''),
          description: '',
          path: URL.createObjectURL(file),
          category: 'Uncategorized',
          addedAt: new Date().toISOString()
        }
        newVideos.push(video)
      }
    })

    if (newVideos.length > 0) {
      const updated = {
        ...library,
        videos: [...newVideos, ...library.videos]
      }
      setLibrary(updated)
      localStorage.setItem('video-library', JSON.stringify(updated))
    }
  }

  const handleDragOver = (event: React.DragEvent) => {
    event.preventDefault()
  }

  return (
    <>
      <Header />
      
      <main className="pt-16 min-h-screen">
        {/* Hero */}
        <section className="border-b border-border bg-bg-secondary">
          <div className="container py-16">
            <h1 className="text-3xl md:text-4xl font-bold mb-4">Video Library</h1>
            <p className="text-text-secondary max-w-2xl">
              Browse research demos, tutorials, and presentations. Drag and drop video files to add them to your local library.
            </p>
          </div>
        </section>

        {/* Controls */}
        <section className="border-b border-border sticky top-16 bg-bg-primary/80 backdrop-blur-xl z-40">
          <div className="container py-4">
            <div className="flex flex-wrap items-center gap-4">
              {/* Search */}
              <div className="relative flex-1 min-w-[200px] max-w-md">
                <svg className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
                <input
                  type="text"
                  placeholder="Search videos..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full pl-10 pr-4 py-2 bg-bg-tertiary border border-border rounded-lg text-sm focus:outline-none focus:border-accent-primary transition-colors"
                />
              </div>

              {/* Category Filter */}
              <select
                value={selectedCategory}
                onChange={(e) => setSelectedCategory(e.target.value)}
                className="px-4 py-2 bg-bg-tertiary border border-border rounded-lg text-sm focus:outline-none focus:border-accent-primary transition-colors"
              >
                <option value="all">All Categories</option>
                {library.categories.map(cat => (
                  <option key={cat} value={cat}>{cat}</option>
                ))}
              </select>

              {/* View Toggle */}
              <div className="flex items-center gap-1 p-1 bg-bg-tertiary rounded-lg">
                <button
                  onClick={() => setViewMode('grid')}
                  className={`p-2 rounded ${viewMode === 'grid' ? 'bg-bg-secondary text-text-primary' : 'text-text-tertiary hover:text-text-secondary'}`}
                >
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
                  </svg>
                </button>
                <button
                  onClick={() => setViewMode('list')}
                  className={`p-2 rounded ${viewMode === 'list' ? 'bg-bg-secondary text-text-primary' : 'text-text-tertiary hover:text-text-secondary'}`}
                >
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M4 6h16M4 12h16M4 18h16" />
                  </svg>
                </button>
              </div>

              {/* Add Videos */}
              <input
                ref={fileInputRef}
                type="file"
                accept="video/*"
                multiple
                onChange={handleFileSelect}
                className="hidden"
              />
              <button
                onClick={() => fileInputRef.current?.click()}
                className="btn btn-primary"
              >
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
                </svg>
                Add Videos
              </button>
            </div>
          </div>
        </section>

        {/* Video Grid */}
        <section 
          className="py-8"
          onDrop={handleDrop}
          onDragOver={handleDragOver}
        >
          <div className="container">
            {filteredVideos.length === 0 ? (
              <div className="py-20 text-center">
                <div className="w-20 h-20 mx-auto mb-6 rounded-2xl bg-bg-tertiary flex items-center justify-center">
                  <svg className="w-10 h-10 text-text-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 10.5l4.72-4.72a.75.75 0 011.28.53v11.38a.75.75 0 01-1.28.53l-4.72-4.72M4.5 18.75h9a2.25 2.25 0 002.25-2.25v-9a2.25 2.25 0 00-2.25-2.25h-9A2.25 2.25 0 002.25 7.5v9a2.25 2.25 0 002.25 2.25z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">No videos yet</h3>
                <p className="text-text-secondary mb-6">
                  Drag and drop video files here, or click &quot;Add Videos&quot; to get started.
                </p>
                <button
                  onClick={() => fileInputRef.current?.click()}
                  className="btn btn-secondary"
                >
                  Browse Files
                </button>
              </div>
            ) : viewMode === 'grid' ? (
              <div className="grid sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                {filteredVideos.map(video => (
                  <div
                    key={video.id}
                    className="group cursor-pointer"
                    onClick={() => setSelectedVideo(video)}
                  >
                    <div className="relative aspect-video bg-bg-tertiary rounded-xl overflow-hidden mb-3">
                      {video.thumbnail ? (
                        <img 
                          src={video.thumbnail} 
                          alt={video.title}
                          className="w-full h-full object-cover"
                        />
                      ) : (
                        <div className="absolute inset-0 flex items-center justify-center">
                          <svg className="w-12 h-12 text-text-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                            <path strokeLinecap="round" strokeLinejoin="round" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            <path strokeLinecap="round" strokeLinejoin="round" d="M15.91 11.672a.375.375 0 010 .656l-5.603 3.113a.375.375 0 01-.557-.328V8.887c0-.286.307-.466.557-.327l5.603 3.112z" />
                          </svg>
                        </div>
                      )}
                      
                      {/* Hover Overlay */}
                      <div className="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                        <div className="w-14 h-14 rounded-full bg-white/20 backdrop-blur-sm flex items-center justify-center">
                          <svg className="w-7 h-7 text-white ml-1" fill="currentColor" viewBox="0 0 24 24">
                            <path d="M8 5v14l11-7z" />
                          </svg>
                        </div>
                      </div>

                      {/* Duration */}
                      {video.duration && (
                        <div className="absolute bottom-2 right-2 px-2 py-0.5 bg-black/70 text-white text-xs font-mono rounded">
                          {video.duration}
                        </div>
                      )}
                    </div>
                    
                    <h3 className="font-medium text-text-primary group-hover:text-accent-primary transition-colors line-clamp-2 mb-1">
                      {video.title}
                    </h3>
                    <div className="flex items-center gap-2 text-xs text-text-tertiary">
                      <span className="tag">{video.category}</span>
                      <span>•</span>
                      <span>{new Date(video.addedAt).toLocaleDateString()}</span>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="space-y-2">
                {filteredVideos.map(video => (
                  <div
                    key={video.id}
                    className="flex items-center gap-4 p-4 bg-bg-secondary rounded-xl hover:bg-bg-tertiary transition-colors cursor-pointer group"
                    onClick={() => setSelectedVideo(video)}
                  >
                    <div className="relative w-40 aspect-video bg-bg-tertiary rounded-lg overflow-hidden flex-shrink-0">
                      {video.thumbnail ? (
                        <img 
                          src={video.thumbnail} 
                          alt={video.title}
                          className="w-full h-full object-cover"
                        />
                      ) : (
                        <div className="absolute inset-0 flex items-center justify-center">
                          <svg className="w-8 h-8 text-text-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                            <path strokeLinecap="round" strokeLinejoin="round" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            <path strokeLinecap="round" strokeLinejoin="round" d="M15.91 11.672a.375.375 0 010 .656l-5.603 3.113a.375.375 0 01-.557-.328V8.887c0-.286.307-.466.557-.327l5.603 3.112z" />
                          </svg>
                        </div>
                      )}
                      {video.duration && (
                        <div className="absolute bottom-1 right-1 px-1.5 py-0.5 bg-black/70 text-white text-xs font-mono rounded">
                          {video.duration}
                        </div>
                      )}
                    </div>
                    
                    <div className="flex-1 min-w-0">
                      <h3 className="font-medium text-text-primary group-hover:text-accent-primary transition-colors truncate mb-1">
                        {video.title}
                      </h3>
                      {video.description && (
                        <p className="text-sm text-text-secondary line-clamp-1 mb-2">
                          {video.description}
                        </p>
                      )}
                      <div className="flex items-center gap-2 text-xs text-text-tertiary">
                        <span className="tag">{video.category}</span>
                        <span>•</span>
                        <span>{new Date(video.addedAt).toLocaleDateString()}</span>
                      </div>
                    </div>

                    <svg className="w-5 h-5 text-text-tertiary group-hover:text-accent-primary transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                      <path strokeLinecap="round" strokeLinejoin="round" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>
      </main>

      {/* Video Modal */}
      {selectedVideo && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/90">
          <div className="w-full max-w-5xl">
            <VideoPlayer
              src={selectedVideo.path}
              title={selectedVideo.title}
              poster={selectedVideo.thumbnail}
              onClose={() => setSelectedVideo(null)}
              autoPlay
            />
          </div>
        </div>
      )}

      <Footer />
    </>
  )
}
