use console::Style;
use std::io::IsTerminal;
use std::sync::OnceLock;

/// Whether color output is enabled globally.
/// Respects NO_COLOR env var, --no-color flag, and TTY detection.
static COLOR_ENABLED: OnceLock<bool> = OnceLock::new();

/// Initialize color support. Call once at startup.
/// After this, `colors_enabled()` returns the cached result.
pub fn init_colors(force_no_color: bool) {
    COLOR_ENABLED.get_or_init(|| {
        if force_no_color {
            return false;
        }
        // Respect the NO_COLOR convention (https://no-color.org/)
        if std::env::var("NO_COLOR").is_ok() {
            return false;
        }
        // Only use colors if stdout is a terminal
        std::io::stdout().is_terminal()
    });
}

/// Returns whether color output is enabled.
pub fn colors_enabled() -> bool {
    *COLOR_ENABLED.get_or_init(|| {
        if std::env::var("NO_COLOR").is_ok() {
            return false;
        }
        std::io::stdout().is_terminal()
    })
}

// ---------------------------------------------------------------------------
// Semantic color system
//
// Every color carries meaning. Nothing is decorative.
//
//   success  = green   -- 2xx status, "saved", "complete", PASS
//   warning  = yellow  -- 3xx/4xx status, degraded state
//   error    = red     -- 5xx status, failures, CRITICAL/HIGH severity
//   info     = blue    -- informational, tips, AI attribution
//   muted    = dim     -- secondary info, timestamps, hints
//   accent   = cyan    -- URLs, commands, interactive elements, JSON keys
//   data     = white   -- response bodies, user content (default)
// ---------------------------------------------------------------------------

/// Style for success indicators (2xx, PASS, saved, complete).
pub fn success() -> Style {
    if colors_enabled() {
        Style::new().green()
    } else {
        Style::new()
    }
}

/// Style for warning indicators (3xx/4xx, degraded).
pub fn warning() -> Style {
    if colors_enabled() {
        Style::new().yellow()
    } else {
        Style::new()
    }
}

/// Style for error indicators (5xx, FAIL, critical).
pub fn error() -> Style {
    if colors_enabled() {
        Style::new().red()
    } else {
        Style::new()
    }
}

/// Style for informational content (tips, AI attribution).
pub fn info() -> Style {
    if colors_enabled() {
        Style::new().blue()
    } else {
        Style::new()
    }
}

/// Style for secondary/muted content (timestamps, hints, labels).
pub fn muted() -> Style {
    if colors_enabled() {
        Style::new().dim()
    } else {
        Style::new()
    }
}

/// Style for accent elements (URLs, commands, JSON keys).
pub fn accent() -> Style {
    if colors_enabled() {
        Style::new().cyan()
    } else {
        Style::new()
    }
}

/// Bold variant of a semantic style.
pub fn success_bold() -> Style {
    if colors_enabled() {
        Style::new().green().bold()
    } else {
        Style::new()
    }
}

pub fn error_bold() -> Style {
    if colors_enabled() {
        Style::new().red().bold()
    } else {
        Style::new()
    }
}

pub fn warning_bold() -> Style {
    if colors_enabled() {
        Style::new().yellow().bold()
    } else {
        Style::new()
    }
}

pub fn info_bold() -> Style {
    if colors_enabled() {
        Style::new().blue().bold()
    } else {
        Style::new()
    }
}

pub fn accent_bold() -> Style {
    if colors_enabled() {
        Style::new().cyan().bold()
    } else {
        Style::new()
    }
}

// ---------------------------------------------------------------------------
// JSON syntax highlighting styles
// ---------------------------------------------------------------------------

/// Style for JSON keys.
pub fn json_key() -> Style {
    if colors_enabled() {
        Style::new().cyan()
    } else {
        Style::new()
    }
}

/// Style for JSON string values.
pub fn json_string() -> Style {
    if colors_enabled() {
        Style::new().green()
    } else {
        Style::new()
    }
}

/// Style for JSON numbers.
pub fn json_number() -> Style {
    if colors_enabled() {
        Style::new().yellow()
    } else {
        Style::new()
    }
}

/// Style for JSON booleans.
pub fn json_bool() -> Style {
    if colors_enabled() {
        Style::new().magenta()
    } else {
        Style::new()
    }
}

/// Style for JSON null.
pub fn json_null() -> Style {
    if colors_enabled() {
        Style::new().red().dim()
    } else {
        Style::new()
    }
}

/// Style for JSON structural characters (braces, brackets, commas, colons).
pub fn json_punct() -> Style {
    if colors_enabled() {
        Style::new().dim()
    } else {
        Style::new()
    }
}
