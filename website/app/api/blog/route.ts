import { NextRequest, NextResponse } from 'next/server'
import { getAllPosts, createPost } from '@/lib/blog'

export async function GET() {
  try {
    const posts = getAllPosts()
    return NextResponse.json(posts)
  } catch (error) {
    console.error('Error fetching posts:', error)
    return NextResponse.json({ error: 'Failed to fetch posts' }, { status: 500 })
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const { title, date, excerpt, author, tags, content } = body

    if (!title || !date || !excerpt || !content) {
      return NextResponse.json(
        { error: 'Missing required fields' },
        { status: 400 }
      )
    }

    const slug = createPost({
      title,
      date,
      excerpt,
      author: author || 'Rogue Intelligence Team',
      tags: tags || [],
      content,
    })

    return NextResponse.json({ slug }, { status: 201 })
  } catch (error) {
    console.error('Error creating post:', error)
    return NextResponse.json({ error: 'Failed to create post' }, { status: 500 })
  }
}
