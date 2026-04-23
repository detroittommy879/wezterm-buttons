//! W111erd-specific TOML config loading for presets, top bar, and gradient
//! Allows customization outside the exe by editing TOML files

use anyhow::Context;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PresetButtonToml {
    pub label: String,
    pub command: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct W111erdPresetsToml {
    pub preset_buttons: Option<Vec<PresetButtonToml>>,
    pub top_bar_buttons: Option<Vec<String>>,
    /// Font size for the left dock and top bar panels (in points).
    /// Defaults to 10 on Windows. Increase to make panel text and buttons larger.
    pub panel_font_size: Option<f64>,
}

/// Reload presets from TOML files. Call this when presets may have changed.
pub fn reload_w111erd_presets() -> Option<W111erdPresetsToml> {
    load_w111erd_presets().ok().flatten()
}

/// Reload gradient from TOML files. Call this when gradient may have changed.
pub fn reload_w111erd_gradient() -> Option<super::BackgroundLayer> {
    load_w111erd_gradient().ok().flatten()
}

pub fn get_w111erd_gradient() -> Option<super::BackgroundLayer> {
    load_w111erd_gradient().ok().flatten()
}

pub fn get_w111erd_presets() -> Option<W111erdPresetsToml> {
    load_w111erd_presets().ok().flatten()
}

fn windows_config_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("APPDATA")
        .map(|s| std::path::PathBuf::from(s).join("wezterm"))
}

fn all_config_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    // On Windows, check APPDATA first
    if let Some(dir) = windows_config_dir() {
        dirs.push(dir);
    }
    // Then add CONFIG_DIRS (which includes XDG_CONFIG_HOME or ~/.config/wezterm)
    for dir in super::CONFIG_DIRS.iter() {
        if !dirs.contains(dir) {
            dirs.push(dir.clone());
        }
    }
    dirs
}

fn load_w111erd_presets() -> anyhow::Result<Option<W111erdPresetsToml>> {
    for dir in all_config_dirs().iter() {
        let path = dir.join("w111erd_presets.toml");
        if path.exists() {
            let contents = std::fs::read_to_string(&path)
                .context(format!("reading w111erd presets from {}", path.display()))?;
            return Ok(Some(toml::from_str(&contents)
                .context(format!("parsing w111erd presets TOML from {}", path.display()))?));
        }
    }
    Ok(None)
}

fn load_w111erd_gradient() -> anyhow::Result<Option<super::BackgroundLayer>> {
    for dir in all_config_dirs().iter() {
        let path = dir.join("w111erd_gradient.toml");
        if path.exists() {
            #[derive(serde::Deserialize)]
            struct GradientToml {
                colors: Vec<String>,
                angle: Option<f64>,
                interpolation: Option<String>,
                blend: Option<String>,
            }

            #[derive(serde::Deserialize)]
            struct W111erdGradientToml {
                gradient: GradientToml,
            }

            let contents = std::fs::read_to_string(&path)
                .context(format!("reading w111erd gradient from {}", path.display()))?;
            let toml: W111erdGradientToml = toml::from_str(&contents)
                .context(format!("parsing w111erd gradient TOML from {}", path.display()))?;

            let interpolation = match toml.gradient.interpolation.as_deref() {
                Some("Linear") => super::Interpolation::Linear,
                Some("CatmullRom") => super::Interpolation::CatmullRom,
                _ => super::Interpolation::Basis,
            };
            let blend = match toml.gradient.blend.as_deref() {
                Some("Rgb") => super::BlendMode::Rgb,
                Some("LinearRgb") => super::BlendMode::LinearRgb,
                Some("Hsv") => super::BlendMode::Hsv,
                _ => super::BlendMode::Oklab,
            };
            let orientation = super::GradientOrientation::Linear { angle: toml.gradient.angle };

            let layer = super::BackgroundLayer {
                source: super::BackgroundSource::Gradient(super::Gradient {
                    orientation,
                    colors: toml.gradient.colors,
                    preset: None,
                    interpolation,
                    blend,
                    segment_size: None,
                    segment_smoothness: None,
                    noise: None,
                }),
                opacity: 1.0,
                hsb: Default::default(),
                origin: Default::default(),
                attachment: Default::default(),
                repeat_x: Default::default(),
                repeat_y: Default::default(),
                repeat_x_size: None,
                repeat_y_size: None,
                vertical_align: Default::default(),
                horizontal_align: Default::default(),
                vertical_offset: None,
                horizontal_offset: None,
                width: super::BackgroundSize::Dimension(super::Dimension::Percent(1.0)),
                height: super::BackgroundSize::Dimension(super::Dimension::Percent(1.0)),
            };
            return Ok(Some(layer));
        }
    }
    Ok(None)
}

/// Returns the preset buttons loaded from TOML, or empty if none loaded
pub fn w111erd_preset_buttons() -> Vec<PresetButtonToml> {
    get_w111erd_presets()
        .and_then(|p| p.preset_buttons)
        .unwrap_or_default()
}

/// Returns the top bar button labels loaded from TOML, or empty if none loaded
pub fn w111erd_top_bar_buttons() -> Vec<String> {
    get_w111erd_presets()
        .and_then(|p| p.top_bar_buttons)
        .unwrap_or_default()
}

/// Default panel font size (points on Windows) used when none is configured
pub const DEFAULT_PANEL_FONT_SIZE: f64 = 10.0;

/// Returns the configured panel font size, or None if not set in TOML
pub fn w111erd_panel_font_size() -> Option<f64> {
    get_w111erd_presets().and_then(|p| p.panel_font_size)
}

/// Returns the scale factor relative to DEFAULT_PANEL_FONT_SIZE.
/// For example: panel_font_size = 14 → scale = 1.4
pub fn w111erd_panel_font_scale() -> f32 {
    w111erd_panel_font_size()
        .map(|size| (size / DEFAULT_PANEL_FONT_SIZE) as f32)
        .unwrap_or(1.0)
}
