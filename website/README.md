# Rogue Intelligence LNC Website

A scientific, professional website for Rogue Intelligence LNC built with Next.js 14.

## Features

- **Scientific Design**: Academic journal-inspired aesthetic with serif typography
- **4 Project Showcases**: Wave, Val-OS, S-wind, Valhalla
- **Blog System**: Markdown-based blog with full CRUD admin panel
- **Admin Dashboard**: Password-protected admin area for content management
- **Responsive**: Works on all device sizes

## Tech Stack

- **Framework**: Next.js 14 (App Router)
- **Styling**: Tailwind CSS
- **Content**: Markdown with gray-matter
- **Typography**: Noto Serif/Sans TC, JetBrains Mono

## Getting Started

### Prerequisites

- Node.js 18+
- npm or yarn

### Installation

```bash
cd website-new
npm install
```

### Development

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000)

### Production Build

```bash
npm run build
npm start
```

## Project Structure

```
website-new/
├── app/                    # Next.js App Router
│   ├── page.tsx           # Homepage
│   ├── layout.tsx         # Root layout
│   ├── globals.css        # Global styles
│   ├── projects/          # Project pages
│   │   ├── wave/
│   │   ├── val-os/
│   │   ├── s-wind/
│   │   └── valhalla/
│   ├── blog/              # Blog pages
│   ├── about/             # About page
│   ├── admin/             # Admin dashboard
│   └── api/               # API routes
├── components/            # React components
├── content/
│   └── blog/             # Markdown blog posts
├── lib/                   # Utility functions
└── public/               # Static assets
```

## Admin Panel

Access the admin panel at `/admin`

**Default Password**: `rogue2026`

(Change this in production by modifying `app/admin/page.tsx`)

### Admin Features

- View all blog posts
- Create new posts
- Edit existing posts
- Delete posts
- Quick links to projects

## Blog Posts

Blog posts are stored as Markdown files in `content/blog/`.

### Post Format

```markdown
---
title: "Post Title"
date: "2026-01-20"
excerpt: "Brief description"
author: "Author Name"
tags: ["tag1", "tag2"]
---

Your content here in Markdown...
```

## Customization

### Colors

Edit `tailwind.config.js` to modify the color palette:

- `paper`: Background tones
- `ink`: Text colors
- `accent`: Primary accent colors

### Typography

Fonts are loaded from Google Fonts in `app/globals.css`.

## Deployment

### Vercel (Recommended)

1. Push to GitHub
2. Import to Vercel
3. Deploy

### Self-Hosted

```bash
npm run build
npm start
```

Or use the standalone output:

```bash
cp -r .next/standalone ./deploy
cp -r .next/static ./deploy/.next/static
cp -r public ./deploy/public
cd deploy
node server.js
```

## License

MIT License - Rogue Intelligence LNC

---

**Rogue Intelligence LNC** - Pioneering the future of adaptive computing.
