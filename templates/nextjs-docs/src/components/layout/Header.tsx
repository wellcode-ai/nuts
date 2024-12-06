import Link from 'next/link'

export function Header() {
  return (
    <header className="bg-cyber-dark/80 backdrop-blur-md border-b border-cyber-primary/20">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          <div className="flex items-center">
            <Link href="/" className="flex items-center space-x-3">
              <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-cyber-primary to-cyber-accent" />
              <h1 className="text-xl cyber-text font-['Orbitron'] animate-glow">
                API Docs
              </h1>
            </Link>
          </div>
          <nav className="flex items-center space-x-4">
            <Link href="/docs" className="btn">
              Documentation
            </Link>
            <Link href="/api-reference" className="btn btn-primary">
              API Reference
            </Link>
          </nav>
        </div>
      </div>
    </header>
  )
} 