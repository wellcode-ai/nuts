import Link from 'next/link'
import { useRouter } from 'next/router'

export function Sidebar() {
  const router = useRouter()
  
  return (
    <aside className="w-64 border-r border-cyber-primary/20 min-h-screen bg-cyber-dark/80 backdrop-blur-md">
      <nav className="p-4 space-y-8">
        <div className="space-y-2">
          <h3 className="text-xs uppercase tracking-wider text-cyber-primary/60 px-4">
            Getting Started
          </h3>
          <ul className="space-y-1">
            <li>
              <Link 
                href="/" 
                className={`block px-4 py-2 rounded-md transition-colors ${
                  router.pathname === '/' 
                    ? 'bg-cyber-primary/10 text-cyber-primary' 
                    : 'hover:bg-cyber-primary/5'
                }`}
              >
                Introduction
              </Link>
            </li>
            <li>
              <Link 
                href="/authentication" 
                className={`block px-4 py-2 rounded-md transition-colors ${
                  router.pathname === '/authentication' 
                    ? 'bg-cyber-primary/10 text-cyber-primary' 
                    : 'hover:bg-cyber-primary/5'
                }`}
              >
                Authentication
              </Link>
            </li>
          </ul>
        </div>

        <div className="space-y-2">
          <h3 className="text-xs uppercase tracking-wider text-cyber-primary/60 px-4">
            API Reference
          </h3>
          <ul className="space-y-1">
            {/* Add your API endpoints here */}
          </ul>
        </div>
      </nav>
    </aside>
  )
} 