import Header from '@/components/Header'
import Footer from '@/components/Footer'
import Link from 'next/link'
import { getAllPosts } from '@/lib/blog'

export const dynamic = 'force-dynamic'

export default function BlogPage() {
  const posts = getAllPosts()

  return (
    <>
      <Header />
      
      <main className="pt-20">
        {/* Hero */}
        <section className="bg-gradient-to-b from-paper-200 to-paper-100 py-16">
          <div className="max-w-4xl mx-auto px-6">
            <h1 className="text-4xl md:text-5xl font-serif font-bold text-ink-900 mb-4">
              Technical Blog
            </h1>
            <p className="text-xl text-ink-600">
              Research updates, technical insights, and development progress from the Rogue Intelligence team.
            </p>
          </div>
        </section>

        {/* Posts */}
        <section className="py-12 bg-paper-100">
          <div className="max-w-4xl mx-auto px-6">
            {posts.length === 0 ? (
              <div className="text-center py-16">
                <p className="text-ink-500 mb-4">No posts yet.</p>
                <p className="text-sm text-ink-400">
                  Check back soon for research updates and technical insights.
                </p>
              </div>
            ) : (
              <div className="space-y-8">
                {posts.map((post) => (
                  <article key={post.slug} className="card">
                    <div className="flex items-center gap-4 text-sm text-ink-500 mb-3">
                      <time className="font-mono">{post.date}</time>
                      <span>•</span>
                      <span>{post.author}</span>
                    </div>
                    <Link href={`/blog/${post.slug}`} className="block border-b-0">
                      <h2 className="text-2xl font-serif font-semibold text-ink-900 mb-3 hover:text-accent-primary transition-colors">
                        {post.title}
                      </h2>
                    </Link>
                    <p className="text-ink-600 mb-4">{post.excerpt}</p>
                    <div className="flex items-center justify-between">
                      <div className="flex flex-wrap gap-2">
                        {post.tags.map((tag) => (
                          <span key={tag} className="academic-tag">{tag}</span>
                        ))}
                      </div>
                      <Link 
                        href={`/blog/${post.slug}`}
                        className="text-sm text-accent-link font-medium hover:underline"
                      >
                        Read more →
                      </Link>
                    </div>
                  </article>
                ))}
              </div>
            )}
          </div>
        </section>
      </main>

      <Footer />
    </>
  )
}
