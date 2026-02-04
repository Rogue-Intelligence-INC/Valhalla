import Header from '@/components/Header'
import Footer from '@/components/Footer'
import Link from 'next/link'
import { getPostBySlug, getAllPosts } from '@/lib/blog'
import { notFound } from 'next/navigation'

export const dynamic = 'force-dynamic'

export async function generateStaticParams() {
  const posts = getAllPosts()
  return posts.map((post) => ({
    slug: post.slug,
  }))
}

export default async function BlogPostPage({ params }: { params: { slug: string } }) {
  const post = await getPostBySlug(params.slug)

  if (!post) {
    notFound()
  }

  return (
    <>
      <Header />
      
      <main className="pt-20">
        {/* Hero */}
        <section className="bg-gradient-to-b from-paper-200 to-paper-100 py-16">
          <div className="max-w-4xl mx-auto px-6">
            <div className="flex items-center gap-2 text-sm text-ink-500 mb-4">
              <Link href="/blog" className="hover:text-accent-primary">Blog</Link>
              <span>/</span>
              <span className="truncate max-w-[200px]">{post.title}</span>
            </div>
            <h1 className="text-3xl md:text-4xl font-serif font-bold text-ink-900 mb-4">
              {post.title}
            </h1>
            <div className="flex items-center gap-4 text-sm text-ink-500">
              <time className="font-mono">{post.date}</time>
              <span>•</span>
              <span>{post.author}</span>
            </div>
            <div className="flex flex-wrap gap-2 mt-4">
              {post.tags.map((tag) => (
                <span key={tag} className="academic-tag">{tag}</span>
              ))}
            </div>
          </div>
        </section>

        {/* Content */}
        <section className="py-12 bg-paper-100">
          <div className="max-w-4xl mx-auto px-6">
            <article 
              className="prose prose-ink max-w-none"
              dangerouslySetInnerHTML={{ __html: post.contentHtml || '' }}
            />

            {/* Back link */}
            <div className="border-t border-paper-300 pt-8 mt-12">
              <Link href="/blog" className="text-accent-link font-medium hover:underline">
                ← Back to all posts
              </Link>
            </div>
          </div>
        </section>
      </main>

      <Footer />
    </>
  )
}
