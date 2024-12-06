/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        cyber: {
          primary: '#0FF4C6',     // Neon cyan
          secondary: '#7B2CBF',   // Deep purple
          accent: '#FF124F',      // Neon pink
          warning: '#FF9E00',     // Neon orange
          dark: '#120458',        // Deep blue
          darker: '#0A0221',      // Darker blue
          light: '#E2E8F0',
          'gray-dark': '#1F2937',
          'gray-light': '#374151',
        },
      },
      backgroundImage: {
        'cyber-grid': "url('/grid-pattern.svg')",
        'cyber-dots': "url('/dots-pattern.svg')",
      },
      boxShadow: {
        'neon': '0 0 5px theme(colors.cyber.primary), 0 0 20px theme(colors.cyber.primary)',
        'neon-lg': '0 0 10px theme(colors.cyber.primary), 0 0 40px theme(colors.cyber.primary)',
        'neon-pink': '0 0 5px theme(colors.cyber.accent), 0 0 20px theme(colors.cyber.accent)',
      },
    },
  },
  plugins: [],
} 