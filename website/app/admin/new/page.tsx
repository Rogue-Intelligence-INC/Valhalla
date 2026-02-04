'use client'

import { useState, useEffect } from 'react'
import { useRouter } from 'next/navigation'
import Link from 'next/link'

export default function NewPostPage() {
  const router = useRouter()
  const [isAuthenticated, setIsAuthenticated] = useState(false)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  
  const [title, setTitle] = useState('')
  const [date, setDate] = useState(new Date().toISOString().split('T')[0])
  const [excerpt, setExcerpt] = useState('')
  const [author, setAuthor] = useState('Rogue Intelligence Team')
  const [tags, setTags] = useState('')
  const [content, setContent] = useState('')

  useEffect(() => {
    const auth = localStorage.getItem('admin_auth')
    if (auth !== 'true') {
      router.push('/admin')
    } else {
      setIsAuthenticated(true)
    }
    setLoading(false)
  }, [router])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setSaving(true)

    try {
      const res = await fetch('/api/blog', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          title,
          date,
          excerpt,
          author,
          tags: tags.split(',').map(t => t.trim()).filter(t => t),
          content,
        }),
      })

      if (res.ok) {
        router.push('/admin')
      } else {
        alert('Failed to create post')
      }
    } catch (err) {
      console.error('Failed to create post:', err)
      alert('Failed to create post')
    } finally {
      setSaving(false)
    }
  }

  if (loading || !isAuthenticated) {
    return (
      <div className="min-h-screen bg-paper-100 flex items-center justify-center">
        <p className="text-ink-500">Loading...</p>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-paper-100">
      {/* Header */}
      <header className="bg-ink-900 text-paper-100 py-4">
        <div className="max-w-4xl mx-auto px-6 flex items-center justify-between">
          <h1 className="text-xl font-serif font-bold">New Blog Post</h1>
          <Link href="/admin" className="text-paper-300 hover:text-paper-100 text-sm">
            ← Back to Dashboard
          </Link>
        </div>
      </header>

      {/* Form */}
      <main className="max-w-4xl mx-auto px-6 py-8">
        <form onSubmit={handleSubmit} className="card">
          <div className="grid md:grid-cols-2 gap-6 mb-6">
            <div>
              <label className="block text-sm font-medium text-ink-700 mb-2">
                Title *
              </label>
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                required
                className="w-full px-4 py-2 border border-paper-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/20 focus:border-accent-primary"
                placeholder="Post title"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-ink-700 mb-2">
                Date *
              </label>
              <input
                type="date"
                value={date}
                onChange={(e) => setDate(e.target.value)}
                required
                className="w-full px-4 py-2 border border-paper-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/20 focus:border-accent-primary"
              />
            </div>
          </div>

          <div className="mb-6">
            <label className="block text-sm font-medium text-ink-700 mb-2">
              Excerpt *
            </label>
            <textarea
              value={excerpt}
              onChange={(e) => setExcerpt(e.target.value)}
              required
              rows={2}
              className="w-full px-4 py-2 border border-paper-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/20 focus:border-accent-primary"
              placeholder="Brief description of the post"
            />
          </div>

          <div className="grid md:grid-cols-2 gap-6 mb-6">
            <div>
              <label className="block text-sm font-medium text-ink-700 mb-2">
                Author
              </label>
              <input
                type="text"
                value={author}
                onChange={(e) => setAuthor(e.target.value)}
                className="w-full px-4 py-2 border border-paper-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/20 focus:border-accent-primary"
                placeholder="Author name"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-ink-700 mb-2">
                Tags (comma-separated)
              </label>
              <input
                type="text"
                value={tags}
                onChange={(e) => setTags(e.target.value)}
                className="w-full px-4 py-2 border border-paper-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/20 focus:border-accent-primary"
                placeholder="Wave, Compiler, Research"
              />
            </div>
          </div>

          <div className="mb-6">
            <label className="block text-sm font-medium text-ink-700 mb-2">
              Content (Markdown) *
            </label>
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              required
              rows={20}
              className="w-full px-4 py-2 border border-paper-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/20 focus:border-accent-primary font-mono text-sm"
              placeholder="Write your post content in Markdown..."
            />
          </div>

          <div className="flex items-center justify-end gap-4">
            <Link href="/admin" className="btn-secondary">
              Cancel
            </Link>
            <button type="submit" disabled={saving} className="btn-primary">
              {saving ? 'Saving...' : 'Publish Post'}
            </button>
          </div>
        </form>
      </main>
    </div>
  )
}
