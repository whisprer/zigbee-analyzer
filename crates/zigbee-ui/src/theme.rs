use serde::{Deserialize, Serialize};

/// Color theme for the TUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
}

/// All color definitions for the theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // UI structure colors
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub border_focused: Color,
    pub title: Color,
    pub status_bar: Color,
    
    // Semantic colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub highlight: Color,
    
    // Traffic/status colors
    pub live: Color,
    pub paused: Color,
    pub inactive: Color,
    
    // Severity levels
    pub severity_critical: Color,
    pub severity_high: Color,
    pub severity_medium: Color,
    pub severity_low: Color,
    pub severity_info: Color,
    
    // Device types
    pub coordinator: Color,
    pub router: Color,
    pub end_device: Color,
    pub unknown_device: Color,
    
    // Signal quality
    pub signal_excellent: Color,
    pub signal_good: Color,
    pub signal_fair: Color,
    pub signal_poor: Color,
    
    // Security
    pub encrypted: Color,
    pub unencrypted: Color,
    pub trusted: Color,
    pub untrusted: Color,
    
    // Data visualization
    pub data_primary: Color,
    pub data_secondary: Color,
    pub data_tertiary: Color,
}

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        }
    }
    
    pub fn to_hex(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
    
    /// Convert to ratatui Color
    pub fn to_ratatui(&self) -> ratatui::style::Color {
        ratatui::style::Color::Rgb(self.r, self.g, self.b)
    }
}

impl Theme {
    /// Default dark theme (cyberpunk style)
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            colors: ThemeColors {
                // UI structure
                background: Color::new(0, 0, 0),
                foreground: Color::new(200, 200, 200),
                border: Color::new(60, 60, 60),
                border_focused: Color::from_hex(0x00D9FF),
                title: Color::from_hex(0x00D9FF),
                status_bar: Color::new(40, 40, 40),
                
                // Semantic
                success: Color::from_hex(0x00FF00),
                warning: Color::from_hex(0xFFAA00),
                error: Color::from_hex(0xFF0000),
                info: Color::from_hex(0x00AAFF),
                highlight: Color::from_hex(0xFF00FF),
                
                // Traffic/status
                live: Color::from_hex(0x00FF00),
                paused: Color::from_hex(0xFFAA00),
                inactive: Color::new(100, 100, 100),
                
                // Severity levels
                severity_critical: Color::from_hex(0xFF0000),
                severity_high: Color::from_hex(0xFF5500),
                severity_medium: Color::from_hex(0xFFAA00),
                severity_low: Color::from_hex(0x00FF00),
                severity_info: Color::from_hex(0x00AAFF),
                
                // Device types
                coordinator: Color::from_hex(0x00FF00),
                router: Color::from_hex(0xFFAA00),
                end_device: Color::from_hex(0x00AAFF),
                unknown_device: Color::new(150, 150, 150),
                
                // Signal quality
                signal_excellent: Color::from_hex(0x00FF00),
                signal_good: Color::from_hex(0xAAFF00),
                signal_fair: Color::from_hex(0xFFAA00),
                signal_poor: Color::from_hex(0xFF0000),
                
                // Security
                encrypted: Color::from_hex(0x00FF00),
                unencrypted: Color::from_hex(0xFF0000),
                trusted: Color::from_hex(0x00AAFF),
                untrusted: Color::from_hex(0xFF5500),
                
                // Data visualization
                data_primary: Color::from_hex(0x00D9FF),
                data_secondary: Color::from_hex(0xFF00FF),
                data_tertiary: Color::from_hex(0xFFAA00),
            },
        }
    }
    
    /// Light theme (for daytime use)
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            colors: ThemeColors {
                // UI structure
                background: Color::new(255, 255, 255),
                foreground: Color::new(0, 0, 0),
                border: Color::new(180, 180, 180),
                border_focused: Color::from_hex(0x0066CC),
                title: Color::from_hex(0x0066CC),
                status_bar: Color::new(240, 240, 240),
                
                // Semantic
                success: Color::from_hex(0x00AA00),
                warning: Color::from_hex(0xFF8800),
                error: Color::from_hex(0xCC0000),
                info: Color::from_hex(0x0066CC),
                highlight: Color::from_hex(0xCC00CC),
                
                // Traffic/status
                live: Color::from_hex(0x00AA00),
                paused: Color::from_hex(0xFF8800),
                inactive: Color::new(150, 150, 150),
                
                // Severity levels
                severity_critical: Color::from_hex(0xCC0000),
                severity_high: Color::from_hex(0xFF4400),
                severity_medium: Color::from_hex(0xFF8800),
                severity_low: Color::from_hex(0x00AA00),
                severity_info: Color::from_hex(0x0066CC),
                
                // Device types
                coordinator: Color::from_hex(0x00AA00),
                router: Color::from_hex(0xFF8800),
                end_device: Color::from_hex(0x0066CC),
                unknown_device: Color::new(120, 120, 120),
                
                // Signal quality
                signal_excellent: Color::from_hex(0x00AA00),
                signal_good: Color::from_hex(0x88CC00),
                signal_fair: Color::from_hex(0xFF8800),
                signal_poor: Color::from_hex(0xCC0000),
                
                // Security
                encrypted: Color::from_hex(0x00AA00),
                unencrypted: Color::from_hex(0xCC0000),
                trusted: Color::from_hex(0x0066CC),
                untrusted: Color::from_hex(0xFF4400),
                
                // Data visualization
                data_primary: Color::from_hex(0x0066CC),
                data_secondary: Color::from_hex(0xCC00CC),
                data_tertiary: Color::from_hex(0xFF8800),
            },
        }
    }
    
    /// Matrix theme (green on black)
    pub fn matrix() -> Self {
        Self {
            name: "Matrix".to_string(),
            colors: ThemeColors {
                // UI structure
                background: Color::new(0, 0, 0),
                foreground: Color::from_hex(0x00FF00),
                border: Color::from_hex(0x00AA00),
                border_focused: Color::from_hex(0x00FF00),
                title: Color::from_hex(0x00FF00),
                status_bar: Color::new(0, 20, 0),
                
                // Semantic
                success: Color::from_hex(0x00FF00),
                warning: Color::from_hex(0x88FF00),
                error: Color::from_hex(0xFF0000),
                info: Color::from_hex(0x00FF88),
                highlight: Color::from_hex(0xAAFF00),
                
                // Traffic/status
                live: Color::from_hex(0x00FF00),
                paused: Color::from_hex(0x88FF00),
                inactive: Color::from_hex(0x004400),
                
                // Severity levels
                severity_critical: Color::from_hex(0xFF0000),
                severity_high: Color::from_hex(0xFF8800),
                severity_medium: Color::from_hex(0xAAFF00),
                severity_low: Color::from_hex(0x00FF00),
                severity_info: Color::from_hex(0x00FF88),
                
                // Device types
                coordinator: Color::from_hex(0x00FF00),
                router: Color::from_hex(0x88FF00),
                end_device: Color::from_hex(0x00FF88),
                unknown_device: Color::from_hex(0x004400),
                
                // Signal quality
                signal_excellent: Color::from_hex(0x00FF00),
                signal_good: Color::from_hex(0x88FF00),
                signal_fair: Color::from_hex(0xAAFF00),
                signal_poor: Color::from_hex(0xFF8800),
                
                // Security
                encrypted: Color::from_hex(0x00FF00),
                unencrypted: Color::from_hex(0xFF0000),
                trusted: Color::from_hex(0x00FF88),
                untrusted: Color::from_hex(0xFF8800),
                
                // Data visualization
                data_primary: Color::from_hex(0x00FF00),
                data_secondary: Color::from_hex(0x88FF00),
                data_tertiary: Color::from_hex(0x00FF88),
            },
        }
    }
    
    /// Solarized Dark theme
    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            colors: ThemeColors {
                background: Color::from_hex(0x002b36),
                foreground: Color::from_hex(0x839496),
                border: Color::from_hex(0x073642),
                border_focused: Color::from_hex(0x268bd2),
                title: Color::from_hex(0x268bd2),
                status_bar: Color::from_hex(0x073642),
                
                success: Color::from_hex(0x859900),
                warning: Color::from_hex(0xb58900),
                error: Color::from_hex(0xdc322f),
                info: Color::from_hex(0x268bd2),
                highlight: Color::from_hex(0xd33682),
                
                live: Color::from_hex(0x859900),
                paused: Color::from_hex(0xb58900),
                inactive: Color::from_hex(0x586e75),
                
                severity_critical: Color::from_hex(0xdc322f),
                severity_high: Color::from_hex(0xcb4b16),
                severity_medium: Color::from_hex(0xb58900),
                severity_low: Color::from_hex(0x859900),
                severity_info: Color::from_hex(0x268bd2),
                
                coordinator: Color::from_hex(0x859900),
                router: Color::from_hex(0xb58900),
                end_device: Color::from_hex(0x268bd2),
                unknown_device: Color::from_hex(0x586e75),
                
                signal_excellent: Color::from_hex(0x859900),
                signal_good: Color::from_hex(0x2aa198),
                signal_fair: Color::from_hex(0xb58900),
                signal_poor: Color::from_hex(0xdc322f),
                
                encrypted: Color::from_hex(0x859900),
                unencrypted: Color::from_hex(0xdc322f),
                trusted: Color::from_hex(0x268bd2),
                untrusted: Color::from_hex(0xcb4b16),
                
                data_primary: Color::from_hex(0x268bd2),
                data_secondary: Color::from_hex(0xd33682),
                data_tertiary: Color::from_hex(0x2aa198),
            },
        }
    }
    
    /// Nord theme
    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            colors: ThemeColors {
                background: Color::from_hex(0x2e3440),
                foreground: Color::from_hex(0xd8dee9),
                border: Color::from_hex(0x3b4252),
                border_focused: Color::from_hex(0x88c0d0),
                title: Color::from_hex(0x88c0d0),
                status_bar: Color::from_hex(0x3b4252),
                
                success: Color::from_hex(0xa3be8c),
                warning: Color::from_hex(0xebcb8b),
                error: Color::from_hex(0xbf616a),
                info: Color::from_hex(0x5e81ac),
                highlight: Color::from_hex(0xb48ead),
                
                live: Color::from_hex(0xa3be8c),
                paused: Color::from_hex(0xebcb8b),
                inactive: Color::from_hex(0x4c566a),
                
                severity_critical: Color::from_hex(0xbf616a),
                severity_high: Color::from_hex(0xd08770),
                severity_medium: Color::from_hex(0xebcb8b),
                severity_low: Color::from_hex(0xa3be8c),
                severity_info: Color::from_hex(0x5e81ac),
                
                coordinator: Color::from_hex(0xa3be8c),
                router: Color::from_hex(0xebcb8b),
                end_device: Color::from_hex(0x5e81ac),
                unknown_device: Color::from_hex(0x4c566a),
                
                signal_excellent: Color::from_hex(0xa3be8c),
                signal_good: Color::from_hex(0x8fbcbb),
                signal_fair: Color::from_hex(0xebcb8b),
                signal_poor: Color::from_hex(0xbf616a),
                
                encrypted: Color::from_hex(0xa3be8c),
                unencrypted: Color::from_hex(0xbf616a),
                trusted: Color::from_hex(0x5e81ac),
                untrusted: Color::from_hex(0xd08770),
                
                data_primary: Color::from_hex(0x88c0d0),
                data_secondary: Color::from_hex(0xb48ead),
                data_tertiary: Color::from_hex(0x8fbcbb),
            },
        }
    }
}

/// Theme manager for loading/saving themes
pub struct ThemeManager {
    current_theme: Theme,
    available_themes: Vec<Theme>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut manager = Self {
            current_theme: Theme::dark(),
            available_themes: Vec::new(),
        };
        
        // Load built-in themes
        manager.available_themes.push(Theme::dark());
        manager.available_themes.push(Theme::light());
        manager.available_themes.push(Theme::matrix());
        manager.available_themes.push(Theme::solarized_dark());
        manager.available_themes.push(Theme::nord());
        
        manager
    }
    
    pub fn current_theme(&self) -> &Theme {
        &self.current_theme
    }
    
    pub fn available_themes(&self) -> &[Theme] {
        &self.available_themes
    }
    
    pub fn set_theme(&mut self, name: &str) -> bool {
        if let Some(theme) = self.available_themes.iter().find(|t| t.name == name) {
            self.current_theme = theme.clone();
            true
        } else {
            false
        }
    }
    
    pub fn load_custom_theme(&mut self, theme: Theme) {
        self.available_themes.push(theme.clone());
        self.current_theme = theme;
    }
    
    /// Save theme to JSON file
    pub fn save_theme(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self.current_theme)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Load theme from JSON file
    pub fn load_theme_from_file(&mut self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let json = std::fs::read_to_string(path)?;
        let theme: Theme = serde_json::from_str(&json)?;
        self.load_custom_theme(theme);
        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to get color for RSSI value
pub fn rssi_color(theme: &Theme, rssi: i8) -> Color {
    if rssi >= -60 {
        theme.colors.signal_excellent
    } else if rssi >= -70 {
        theme.colors.signal_good
    } else if rssi >= -80 {
        theme.colors.signal_fair
    } else {
        theme.colors.signal_poor
    }
}

/// Helper to get color for LQI value
pub fn lqi_color(theme: &Theme, lqi: u8) -> Color {
    if lqi >= 200 {
        theme.colors.signal_excellent
    } else if lqi >= 150 {
        theme.colors.signal_good
    } else if lqi >= 100 {
        theme.colors.signal_fair
    } else {
        theme.colors.signal_poor
    }
}

/// Helper to get color for encryption status
pub fn encryption_color(theme: &Theme, encrypted: bool) -> Color {
    if encrypted {
        theme.colors.encrypted
    } else {
        theme.colors.unencrypted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_color_hex_conversion() {
        let color = Color::from_hex(0xFF8800);
        assert_eq!(color.r, 0xFF);
        assert_eq!(color.g, 0x88);
        assert_eq!(color.b, 0x00);
        assert_eq!(color.to_hex(), 0xFF8800);
    }
    
    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();
        assert_eq!(manager.current_theme().name, "Dark");
        
        manager.set_theme("Matrix");
        assert_eq!(manager.current_theme().name, "Matrix");
        
        assert!(manager.available_themes().len() >= 5);
    }
    
    #[test]
    fn test_rssi_color() {
        let theme = Theme::dark();
        
        let color = rssi_color(&theme, -50);
        assert_eq!(color, theme.colors.signal_excellent);
        
        let color = rssi_color(&theme, -90);
        assert_eq!(color, theme.colors.signal_poor);
    }
}