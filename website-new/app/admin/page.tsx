'use client'

import { useState, useEffect } from 'react'
import Header from '@/components/Header'
import { 
  loadSiteContent, 
  saveSiteContent, 
  resetSiteContent,
  loadVideoLibrary,
  saveVideoLibrary,
  SiteContent,
  VideoLibrary,
  VideoItem,
  defaultSiteContent
} from '@/lib/content'

type Tab = 'content' | 'videos' | 'settings'

export default function AdminPage() {
  const [activeTab, setActiveTab] = useState<Tab>('content')
  const [content, setContent] = useState<SiteContent>(defaultSiteContent)
  const [videoLibrary, setVideoLibrary] = useState<VideoLibrary>({ videos: [], categories: [] })
  const [saved, setSaved] = useState(false)
  const [editingVideo, setEditingVideo] = useState<VideoItem | null>(null)
  const [newCategory, setNewCategory] = useState('')

  useEffect(() => {
    setContent(loadSiteContent())
    setVideoLibrary(loadVideoLibrary())
  }, [])

  const handleSaveContent = () => {
    saveSiteContent(content)
    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }

  const handleResetContent = () => {
    if (confirm('Reset all content to defaults? This cannot be undone.')) {
      const defaultContent = resetSiteContent()
      setContent(defaultContent)
    }
  }

  const handleUpdateHero = (field: keyof SiteContent['hero'], value: string) => {
    setContent(prev => ({
      ...prev,
      hero: { ...prev.hero, [field]: value }
    }))
  }

  const handleUpdateMetric = (index: number, field: 'label' | 'value', value: string) => {
    setContent(prev => ({
      ...prev,
      metrics: prev.metrics.map((m, i) => i === index ? { ...m, [field]: value } : m)
    }))
  }

  const handleUpdatePhilosophy = (index: number, field: 'title' | 'description', value: string) => {
    setContent(prev => ({
      ...prev,
      philosophy: {
        ...prev.philosophy,
        items: prev.philosophy.items.map((item, i) => 
          i === index ? { ...item, [field]: value } : item
        )
      }
    }))
  }

  const handleDeleteVideo = (id: string) => {
    if (confirm('Delete this video?')) {
      const updated = {
        ...videoLibrary,
        videos: videoLibrary.videos.filter(v => v.id !== id)
      }
      setVideoLibrary(updated)
      saveVideoLibrary(updated)
    }
  }

  const handleUpdateVideo = (video: VideoItem) => {
    const updated = {
      ...videoLibrary,
      videos: videoLibrary.videos.map(v => v.id === video.id ? video : v)
    }
    setVideoLibrary(updated)
    saveVideoLibrary(updated)
    setEditingVideo(null)
  }

  const handleAddCategory = () => {
    if (newCategory && !videoLibrary.categories.includes(newCategory)) {
      const updated = {
        ...videoLibrary,
        categories: [...videoLibrary.categories, newCategory]
      }
      setVideoLibrary(updated)
      saveVideoLibrary(updated)
      setNewCategory('')
    }
  }

  const handleDeleteCategory = (cat: string) => {
    const updated = {
      ...videoLibrary,
      categories: videoLibrary.categories.filter(c => c !== cat)
    }
    setVideoLibrary(updated)
    saveVideoLibrary(updated)
  }

  return (
    <>
      <Header />
      
      <main className="pt-16 min-h-screen bg-bg-primary">
        <div className="container py-8">
          <div className="flex items-center justify-between mb-8">
            <div>
              <h1 className="text-2xl font-bold mb-1">Admin Dashboard</h1>
              <p className="text-text-secondary text-sm">Manage site content and videos</p>
            </div>
            {content.updatedAt && (
              <p className="text-xs text-text-tertiary">
                Last updated: {new Date(content.updatedAt).toLocaleString()}
              </p>
            )}
          </div>

          {/* Tabs */}
          <div className="flex gap-1 p-1 bg-bg-tertiary rounded-lg w-fit mb-8">
            {(['content', 'videos', 'settings'] as Tab[]).map(tab => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  activeTab === tab 
                    ? 'bg-bg-secondary text-text-primary shadow-sm' 
                    : 'text-text-secondary hover:text-text-primary'
                }`}
              >
                {tab.charAt(0).toUpperCase() + tab.slice(1)}
              </button>
            ))}
          </div>

          {/* Content Tab */}
          {activeTab === 'content' && (
            <div className="space-y-8">
              {/* Hero Section */}
              <section className="card">
                <h2 className="text-lg font-semibold mb-6">Hero Section</h2>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-text-secondary mb-1.5">
                      Tagline
                    </label>
                    <input
                      type="text"
                      value={content.hero.tagline}
                      onChange={(e) => handleUpdateHero('tagline', e.target.value)}
                      className="w-full px-4 py-2.5 bg-bg-tertiary border border-border rounded-lg focus:outline-none focus:border-accent-primary transition-colors"
                    />
                  </div>
                  <div className="grid md:grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm font-medium text-text-secondary mb-1.5">
                        Title
                      </label>
                      <input
                        type="text"
                        value={content.hero.title}
                        onChange={(e) => handleUpdateHero('title', e.target.value)}
                        className="w-full px-4 py-2.5 bg-bg-tertiary border border-border rounded-lg focus:outline-none focus:border-accent-primary transition-colors"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-text-secondary mb-1.5">
                        Subtitle
                      </label>
                      <input
                        type="text"
                        value={content.hero.subtitle}
                        onChange={(e) => handleUpdateHero('subtitle', e.target.value)}
                        className="w-full px-4 py-2.5 bg-bg-tertiary border border-border rounded-lg focus:outline-none focus:border-accent-primary transition-colors"
                      />
                    </div>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-text-secondary mb-1.5">
                      Description
                    </label>
                    <textarea
                      value={content.hero.description}
                      onChange={(e) => handleUpdateHero('description', e.target.value)}
                      rows={3}
                      className="w-full px-4 py-2.5 bg-bg-tertiary border border-border rounded-lg focus:outline-none focus:border-accent-primary transition-colors resize-none"
                    />
                  </div>
                </div>
              </section>

              {/* Metrics Section */}
              <section className="card">
                <h2 className="text-lg font-semibold mb-6">Metrics</h2>
                <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-4">
                  {content.metrics.map((metric, index) => (
                    <div key={index} className="p-4 bg-bg-tertiary rounded-lg">
                      <input
                        type="text"
                        value={metric.value}
                        onChange={(e) => handleUpdateMetric(index, 'value', e.target.value)}
                        className="w-full text-2xl font-bold bg-transparent border-none focus:outline-none text-center mb-2"
                        placeholder="Value"
                      />
                      <input
                        type="text"
                        value={metric.label}
                        onChange={(e) => handleUpdateMetric(index, 'label', e.target.value)}
                        className="w-full text-sm text-text-secondary bg-transparent border-none focus:outline-none text-center"
                        placeholder="Label"
                      />
                    </div>
                  ))}
                </div>
              </section>

              {/* Philosophy Section */}
              <section className="card">
                <div className="flex items-center justify-between mb-6">
                  <h2 className="text-lg font-semibold">Philosophy</h2>
                  <input
                    type="text"
                    value={content.philosophy.title}
                    onChange={(e) => setContent(prev => ({
                      ...prev,
                      philosophy: { ...prev.philosophy, title: e.target.value }
                    }))}
                    className="px-3 py-1.5 bg-bg-tertiary border border-border rounded-lg text-sm focus:outline-none focus:border-accent-primary"
                    placeholder="Section title"
                  />
                </div>
                <div className="space-y-4">
                  {content.philosophy.items.map((item, index) => (
                    <div key={index} className="flex gap-4 p-4 bg-bg-tertiary rounded-lg">
                      <div className="w-10 h-10 rounded-xl bg-accent-primary/10 flex items-center justify-center flex-shrink-0">
                        <span className="text-accent-primary font-semibold">{index + 1}</span>
                      </div>
                      <div className="flex-1 space-y-2">
                        <input
                          type="text"
                          value={item.title}
                          onChange={(e) => handleUpdatePhilosophy(index, 'title', e.target.value)}
                          className="w-full font-medium bg-transparent border-none focus:outline-none"
                          placeholder="Title"
                        />
                        <input
                          type="text"
                          value={item.description}
                          onChange={(e) => handleUpdatePhilosophy(index, 'description', e.target.value)}
                          className="w-full text-sm text-text-secondary bg-transparent border-none focus:outline-none"
                          placeholder="Description"
                        />
                      </div>
                    </div>
                  ))}
                </div>
              </section>

              {/* Actions */}
              <div className="flex items-center gap-4">
                <button
                  onClick={handleSaveContent}
                  className="btn btn-primary"
                >
                  {saved ? (
                    <>
                      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                      </svg>
                      Saved!
                    </>
                  ) : (
                    <>
                      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
                      </svg>
                      Save Changes
                    </>
                  )}
                </button>
                <button
                  onClick={handleResetContent}
                  className="btn btn-ghost text-accent-danger"
                >
                  Reset to Defaults
                </button>
              </div>
            </div>
          )}

          {/* Videos Tab */}
          {activeTab === 'videos' && (
            <div className="space-y-8">
              {/* Categories */}
              <section className="card">
                <h2 className="text-lg font-semibold mb-4">Categories</h2>
                <div className="flex flex-wrap gap-2 mb-4">
                  {videoLibrary.categories.map(cat => (
                    <span 
                      key={cat}
                      className="inline-flex items-center gap-2 px-3 py-1.5 bg-bg-tertiary rounded-lg text-sm"
                    >
                      {cat}
                      <button
                        onClick={() => handleDeleteCategory(cat)}
                        className="text-text-tertiary hover:text-accent-danger transition-colors"
                      >
                        <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                          <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </span>
                  ))}
                </div>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={newCategory}
                    onChange={(e) => setNewCategory(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleAddCategory()}
                    placeholder="New category..."
                    className="flex-1 px-4 py-2 bg-bg-tertiary border border-border rounded-lg text-sm focus:outline-none focus:border-accent-primary"
                  />
                  <button
                    onClick={handleAddCategory}
                    className="btn btn-secondary"
                  >
                    Add
                  </button>
                </div>
              </section>

              {/* Video List */}
              <section className="card">
                <h2 className="text-lg font-semibold mb-4">
                  Videos ({videoLibrary.videos.length})
                </h2>
                
                {videoLibrary.videos.length === 0 ? (
                  <p className="text-text-secondary text-sm py-8 text-center">
                    No videos yet. Add videos from the Videos page.
                  </p>
                ) : (
                  <div className="space-y-3">
                    {videoLibrary.videos.map(video => (
                      <div 
                        key={video.id}
                        className="flex items-center gap-4 p-4 bg-bg-tertiary rounded-lg"
                      >
                        <div className="w-24 aspect-video bg-bg-secondary rounded-lg overflow-hidden flex-shrink-0">
                          {video.thumbnail ? (
                            <img src={video.thumbnail} alt="" className="w-full h-full object-cover" />
                          ) : (
                            <div className="w-full h-full flex items-center justify-center">
                              <svg className="w-6 h-6 text-text-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15.75 10.5l4.72-4.72a.75.75 0 011.28.53v11.38a.75.75 0 01-1.28.53l-4.72-4.72M4.5 18.75h9a2.25 2.25 0 002.25-2.25v-9a2.25 2.25 0 00-2.25-2.25h-9A2.25 2.25 0 002.25 7.5v9a2.25 2.25 0 002.25 2.25z" />
                              </svg>
                            </div>
                          )}
                        </div>
                        
                        {editingVideo?.id === video.id ? (
                          <div className="flex-1 space-y-2">
                            <input
                              type="text"
                              value={editingVideo.title}
                              onChange={(e) => setEditingVideo({ ...editingVideo, title: e.target.value })}
                              className="w-full px-3 py-1.5 bg-bg-secondary border border-border rounded text-sm focus:outline-none focus:border-accent-primary"
                              placeholder="Title"
                            />
                            <input
                              type="text"
                              value={editingVideo.description}
                              onChange={(e) => setEditingVideo({ ...editingVideo, description: e.target.value })}
                              className="w-full px-3 py-1.5 bg-bg-secondary border border-border rounded text-sm focus:outline-none focus:border-accent-primary"
                              placeholder="Description"
                            />
                            <select
                              value={editingVideo.category}
                              onChange={(e) => setEditingVideo({ ...editingVideo, category: e.target.value })}
                              className="px-3 py-1.5 bg-bg-secondary border border-border rounded text-sm focus:outline-none focus:border-accent-primary"
                            >
                              {videoLibrary.categories.map(cat => (
                                <option key={cat} value={cat}>{cat}</option>
                              ))}
                            </select>
                            <div className="flex gap-2">
                              <button
                                onClick={() => handleUpdateVideo(editingVideo)}
                                className="btn btn-primary text-xs py-1.5"
                              >
                                Save
                              </button>
                              <button
                                onClick={() => setEditingVideo(null)}
                                className="btn btn-ghost text-xs py-1.5"
                              >
                                Cancel
                              </button>
                            </div>
                          </div>
                        ) : (
                          <>
                            <div className="flex-1 min-w-0">
                              <h3 className="font-medium truncate">{video.title}</h3>
                              <p className="text-sm text-text-secondary truncate">{video.description || 'No description'}</p>
                              <div className="flex items-center gap-2 mt-1">
                                <span className="tag text-xs">{video.category}</span>
                                <span className="text-xs text-text-tertiary">
                                  {new Date(video.addedAt).toLocaleDateString()}
                                </span>
                              </div>
                            </div>
                            <div className="flex items-center gap-2">
                              <button
                                onClick={() => setEditingVideo(video)}
                                className="p-2 text-text-secondary hover:text-text-primary hover:bg-bg-secondary rounded-lg transition-colors"
                              >
                                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                                  <path strokeLinecap="round" strokeLinejoin="round" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                </svg>
                              </button>
                              <button
                                onClick={() => handleDeleteVideo(video.id)}
                                className="p-2 text-text-secondary hover:text-accent-danger hover:bg-bg-secondary rounded-lg transition-colors"
                              >
                                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                                  <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                </svg>
                              </button>
                            </div>
                          </>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </section>
            </div>
          )}

          {/* Settings Tab */}
          {activeTab === 'settings' && (
            <div className="space-y-8">
              <section className="card">
                <h2 className="text-lg font-semibold mb-4">Export / Import</h2>
                <p className="text-sm text-text-secondary mb-4">
                  Export your content and video library settings as JSON, or import from a backup.
                </p>
                <div className="flex gap-4">
                  <button
                    onClick={() => {
                      const data = {
                        content: loadSiteContent(),
                        videos: loadVideoLibrary(),
                        exportedAt: new Date().toISOString()
                      }
                      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
                      const url = URL.createObjectURL(blob)
                      const a = document.createElement('a')
                      a.href = url
                      a.download = `rogue-intelligence-backup-${new Date().toISOString().split('T')[0]}.json`
                      a.click()
                    }}
                    className="btn btn-secondary"
                  >
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                    </svg>
                    Export Backup
                  </button>
                  <label className="btn btn-secondary cursor-pointer">
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                    </svg>
                    Import Backup
                    <input
                      type="file"
                      accept=".json"
                      className="hidden"
                      onChange={(e) => {
                        const file = e.target.files?.[0]
                        if (file) {
                          const reader = new FileReader()
                          reader.onload = (event) => {
                            try {
                              const data = JSON.parse(event.target?.result as string)
                              if (data.content) {
                                saveSiteContent(data.content)
                                setContent(data.content)
                              }
                              if (data.videos) {
                                saveVideoLibrary(data.videos)
                                setVideoLibrary(data.videos)
                              }
                              alert('Backup imported successfully!')
                            } catch (err) {
                              alert('Failed to import backup. Invalid file format.')
                            }
                          }
                          reader.readAsText(file)
                        }
                      }}
                    />
                  </label>
                </div>
              </section>

              <section className="card border-accent-danger/30">
                <h2 className="text-lg font-semibold text-accent-danger mb-4">Danger Zone</h2>
                <p className="text-sm text-text-secondary mb-4">
                  Clear all stored data including content and video library. This cannot be undone.
                </p>
                <button
                  onClick={() => {
                    if (confirm('Are you sure? This will delete all your content and video data.')) {
                      localStorage.removeItem('site-content')
                      localStorage.removeItem('video-library')
                      setContent(defaultSiteContent)
                      setVideoLibrary({ videos: [], categories: ['Research', 'Demos', 'Tutorials', 'Presentations'] })
                      alert('All data cleared.')
                    }
                  }}
                  className="btn bg-accent-danger text-white hover:bg-red-600"
                >
                  Clear All Data
                </button>
              </section>
            </div>
          )}
        </div>
      </main>
    </>
  )
}
