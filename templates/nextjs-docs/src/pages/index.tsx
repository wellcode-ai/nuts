import { apiConfig } from '@/config/api-config'
import { motion } from 'framer-motion'

export default function Home() {
  return (
    <div className="space-y-8">
      <motion.div 
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="space-y-4"
      >
        <h1 className="text-4xl font-bold">{apiConfig.title}</h1>
        <p className="text-lg text-gray-600">Version {apiConfig.version}</p>
      </motion.div>

      <div className="grid gap-6 md:grid-cols-2">
        <motion.div 
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="card"
        >
          <h2 className="text-xl font-semibold mb-4">Getting Started</h2>
          <p className="text-gray-600">
            Welcome to the API documentation. Get started by exploring our endpoints
            or reading through the guides.
          </p>
        </motion.div>

        <motion.div 
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="card"
        >
          <h2 className="text-xl font-semibold mb-4">Base URL</h2>
          <code className="block p-4 bg-gray-50 rounded-md">
            {apiConfig.baseUrl}
          </code>
        </motion.div>
      </div>
    </div>
  )
} 