export default function Home() {
  return (
    <div className="flash-bg min-h-screen flex flex-col items-center justify-center relative px-4">
      {/* Floating particles */}
      <div className="particle"></div>
      <div className="particle"></div>
      <div className="particle"></div>
      <div className="particle"></div>
      <div className="particle"></div>

      {/* Main content */}
      <div className="text-center mb-12">
        {/* Pop NUTS logo */}
        <h1 className="nuts-logo mb-4">
          NUTS
        </h1>
        
        {/* Clear subtitle */}
        <p className="subtitle">
          AI-Powered CURL Killer & API Testing Revolution
        </p>
      </div>

      {/* Mini terminal */}
      <div className="mini-terminal">
        {/* Terminal title bar */}
        <div className="terminal-titlebar">
          <div className="terminal-button btn-red"></div>
          <div className="terminal-button btn-yellow"></div>
          <div className="terminal-button btn-green"></div>
          <div className="terminal-title">Terminal</div>
        </div>
        
        {/* Terminal content */}
        <div className="terminal-content">
          <div className="terminal-line">
            <span className="terminal-prompt">$</span>
            <span className="terminal-command">cargo install --git https://github.com/wellcode-ai/nuts</span>
          </div>
          
          <div className="terminal-line">
            <span className="terminal-prompt">$</span>
            <span className="terminal-command">nuts ask &quot;Create 5 test users with realistic data&quot;</span>
            <span className="terminal-comment"># AI CURL killer</span>
          </div>
          
          <div className="terminal-line">
            <span className="terminal-prompt">$</span>
            <span className="terminal-command">nuts generate products 50</span>
            <span className="terminal-comment"># AI test data</span>
          </div>
          
          <div className="terminal-line">
            <span className="terminal-prompt">$</span>
            <span className="terminal-command">nuts monitor https://api.myapp.com --smart</span>
            <span className="terminal-comment"># Smart monitoring</span>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <div className="absolute bottom-16 text-center w-full">
        <div className="flex justify-center gap-4 mb-4">
          <a href="/readme" className="cyberpunk-button">
            📖 README
          </a>
          <a href="https://github.com/wellcode-ai/nuts" className="cyberpunk-button">
            🚀 GitHub
          </a>
        </div>
      </div>

      {/* Footer */}
      <div className="absolute bottom-8 text-center">
        <p className="text-white text-sm font-medium opacity-80">
          Built with ❤️ by WellCode AI
        </p>
      </div>
    </div>
  );
}