'use client'

import { useEffect, useRef } from 'react'

interface Point {
  x: number
  y: number
  originY: number
  speed: number
  amplitude: number
  phase: number
  offset: number
}

export default function WaveBackground() {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const mouseRef = useRef({ x: 0, y: 0, targetX: 0, targetY: 0 })
  const pointsRef = useRef<Point[][]>([])
  const timeRef = useRef(0)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    let animationId: number

    // Set canvas size
    const setSize = () => {
      canvas.width = window.innerWidth
      canvas.height = window.innerHeight
      initPoints()
    }

    // Initialize wave points
    const initPoints = () => {
      const waveCount = 6
      const pointCount = Math.ceil(canvas.width / 30) + 1
      pointsRef.current = []

      for (let w = 0; w < waveCount; w++) {
        const points: Point[] = []
        const baseY = canvas.height * (0.2 + w * 0.13)
        
        for (let i = 0; i < pointCount; i++) {
          points.push({
            x: i * 30,
            y: baseY,
            originY: baseY,
            speed: 0.015 + Math.random() * 0.01,
            amplitude: 15 + Math.random() * 25 + w * 5,
            phase: Math.random() * Math.PI * 2,
            offset: i * 0.15 + w * 0.5,
          })
        }
        pointsRef.current.push(points)
      }
    }

    // Mouse move handler
    const handleMouseMove = (e: MouseEvent) => {
      mouseRef.current.targetX = e.clientX
      mouseRef.current.targetY = e.clientY
    }

    // Smooth mouse position
    const updateMouse = () => {
      const mouse = mouseRef.current
      mouse.x += (mouse.targetX - mouse.x) * 0.05
      mouse.y += (mouse.targetY - mouse.y) * 0.05
    }

    // Perlin-like noise function
    const noise = (x: number, y: number, t: number) => {
      return Math.sin(x * 0.01 + t) * Math.cos(y * 0.01 + t * 0.7) * 
             Math.sin((x + y) * 0.005 + t * 0.5)
    }

    // Animation loop
    const animate = () => {
      timeRef.current += 0.008
      updateMouse()

      // Clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height)

      const mouse = mouseRef.current
      const time = timeRef.current

      // Draw each wave
      pointsRef.current.forEach((points, waveIndex) => {
        // Update points
        points.forEach((point, i) => {
          // Multiple layered sine waves for natural motion
          const wave1 = Math.sin(time * 1.2 + point.offset) * point.amplitude * 0.6
          const wave2 = Math.sin(time * 0.8 + point.offset * 1.5 + point.phase) * point.amplitude * 0.3
          const wave3 = Math.sin(time * 2.1 + point.offset * 0.7) * point.amplitude * 0.15
          const naturalWave = wave1 + wave2 + wave3

          // Perlin-like noise for organic movement
          const noiseValue = noise(point.x, point.originY, time) * 20

          // Mouse influence - creates ripple effect
          const dx = mouse.x - point.x
          const dy = mouse.y - point.originY
          const distance = Math.sqrt(dx * dx + dy * dy)
          const maxDistance = 350
          const influence = Math.max(0, 1 - distance / maxDistance)
          const mouseWave = influence * influence * 80 * Math.sin(distance * 0.015 - time * 3)

          // Secondary mouse wave for smoother effect
          const mouseWave2 = influence * 40 * Math.cos(distance * 0.02 - time * 2)

          // Combine all effects
          point.y = point.originY + naturalWave + noiseValue + mouseWave + mouseWave2 * 0.5
        })

        // Draw smooth bezier curve through points
        ctx.beginPath()
        
        if (points.length < 2) return

        // Start from first point
        ctx.moveTo(points[0].x, points[0].y)

        // Draw smooth curve using bezier
        for (let i = 0; i < points.length - 1; i++) {
          const current = points[i]
          const next = points[i + 1]
          
          // Control point for smooth curve
          const cpX = (current.x + next.x) / 2
          const cpY = (current.y + next.y) / 2
          
          ctx.quadraticCurveTo(current.x, current.y, cpX, cpY)
        }

        // Connect to last point
        const lastPoint = points[points.length - 1]
        ctx.lineTo(lastPoint.x, lastPoint.y)

        // Style based on wave index - gradient opacity
        const baseOpacity = 0.04
        const opacity = baseOpacity + waveIndex * 0.025
        const isDark = document.documentElement.classList.contains('dark')
        
        ctx.strokeStyle = isDark 
          ? `rgba(255, 255, 255, ${opacity})` 
          : `rgba(0, 0, 0, ${opacity})`
        ctx.lineWidth = 1.2 + (pointsRef.current.length - waveIndex) * 0.1
        ctx.lineCap = 'round'
        ctx.lineJoin = 'round'
        ctx.stroke()
      })

      animationId = requestAnimationFrame(animate)
    }

    // Initialize
    setSize()
    window.addEventListener('resize', setSize)
    window.addEventListener('mousemove', handleMouseMove)
    
    // Start with mouse in center
    mouseRef.current = {
      x: window.innerWidth / 2,
      y: window.innerHeight / 2,
      targetX: window.innerWidth / 2,
      targetY: window.innerHeight / 2,
    }

    animationId = requestAnimationFrame(animate)

    return () => {
      window.removeEventListener('resize', setSize)
      window.removeEventListener('mousemove', handleMouseMove)
      cancelAnimationFrame(animationId)
    }
  }, [])

  return (
    <canvas
      ref={canvasRef}
      className="fixed inset-0 pointer-events-none"
      style={{ zIndex: 1 }}
    />
  )
}
