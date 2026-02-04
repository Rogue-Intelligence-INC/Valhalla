import fs from 'fs'
import path from 'path'
import matter from 'gray-matter'
import { remark } from 'remark'
import html from 'remark-html'

const postsDirectory = path.join(process.cwd(), 'content/blog')

export interface BlogPost {
  slug: string
  title: string
  date: string
  excerpt: string
  author: string
  tags: string[]
  content: string
  contentHtml?: string
}

// Ensure directory exists
function ensureDirectoryExists() {
  if (!fs.existsSync(postsDirectory)) {
    fs.mkdirSync(postsDirectory, { recursive: true })
  }
}

export function getAllPosts(): BlogPost[] {
  ensureDirectoryExists()
  
  const fileNames = fs.readdirSync(postsDirectory).filter(f => f.endsWith('.md'))
  
  const allPosts = fileNames.map((fileName) => {
    const slug = fileName.replace(/\.md$/, '')
    const fullPath = path.join(postsDirectory, fileName)
    const fileContents = fs.readFileSync(fullPath, 'utf8')
    const matterResult = matter(fileContents)

    return {
      slug,
      title: matterResult.data.title || 'Untitled',
      date: matterResult.data.date || new Date().toISOString().split('T')[0],
      excerpt: matterResult.data.excerpt || '',
      author: matterResult.data.author || 'Rogue Intelligence Team',
      tags: matterResult.data.tags || [],
      content: matterResult.content,
    }
  })

  return allPosts.sort((a, b) => (a.date < b.date ? 1 : -1))
}

export async function getPostBySlug(slug: string): Promise<BlogPost | null> {
  ensureDirectoryExists()
  
  const fullPath = path.join(postsDirectory, `${slug}.md`)
  
  if (!fs.existsSync(fullPath)) {
    return null
  }

  const fileContents = fs.readFileSync(fullPath, 'utf8')
  const matterResult = matter(fileContents)

  const processedContent = await remark()
    .use(html)
    .process(matterResult.content)
  const contentHtml = processedContent.toString()

  return {
    slug,
    title: matterResult.data.title || 'Untitled',
    date: matterResult.data.date || new Date().toISOString().split('T')[0],
    excerpt: matterResult.data.excerpt || '',
    author: matterResult.data.author || 'Rogue Intelligence Team',
    tags: matterResult.data.tags || [],
    content: matterResult.content,
    contentHtml,
  }
}

export function createPost(post: Omit<BlogPost, 'slug' | 'contentHtml'>): string {
  ensureDirectoryExists()
  
  // Generate slug from title
  const slug = post.title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '')

  const frontmatter = `---
title: "${post.title}"
date: "${post.date}"
excerpt: "${post.excerpt}"
author: "${post.author}"
tags: [${post.tags.map(t => `"${t}"`).join(', ')}]
---

${post.content}`

  const fullPath = path.join(postsDirectory, `${slug}.md`)
  fs.writeFileSync(fullPath, frontmatter)
  
  return slug
}

export function updatePost(slug: string, post: Partial<BlogPost>): boolean {
  ensureDirectoryExists()
  
  const fullPath = path.join(postsDirectory, `${slug}.md`)
  
  if (!fs.existsSync(fullPath)) {
    return false
  }

  const fileContents = fs.readFileSync(fullPath, 'utf8')
  const matterResult = matter(fileContents)

  const updatedData = {
    ...matterResult.data,
    ...(post.title && { title: post.title }),
    ...(post.date && { date: post.date }),
    ...(post.excerpt && { excerpt: post.excerpt }),
    ...(post.author && { author: post.author }),
    ...(post.tags && { tags: post.tags }),
  }

  const updatedContent = post.content !== undefined ? post.content : matterResult.content

  const newFrontmatter = `---
title: "${updatedData.title}"
date: "${updatedData.date}"
excerpt: "${updatedData.excerpt}"
author: "${updatedData.author}"
tags: [${(updatedData.tags || []).map((t: string) => `"${t}"`).join(', ')}]
---

${updatedContent}`

  fs.writeFileSync(fullPath, newFrontmatter)
  return true
}

export function deletePost(slug: string): boolean {
  ensureDirectoryExists()
  
  const fullPath = path.join(postsDirectory, `${slug}.md`)
  
  if (!fs.existsSync(fullPath)) {
    return false
  }

  fs.unlinkSync(fullPath)
  return true
}
