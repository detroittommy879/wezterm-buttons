use crate::quad::TripleLayerQuadAllocator;
use crate::termwindow::box_model::{
    BorderColor, BoxDimension, Corners, DisplayType, Element, ElementColors, ElementContent,
    LayoutContext, SizedPoly, VerticalAlign,
};
use crate::termwindow::render::corners::{
    BOTTOM_LEFT_ROUNDED_CORNER, BOTTOM_RIGHT_ROUNDED_CORNER, TOP_LEFT_ROUNDED_CORNER,
    TOP_RIGHT_ROUNDED_CORNER,
};
use crate::termwindow::{TopBarItem, UIItem};
use config::{w111erd_config, Dimension, TabBarColors};
use lazy_static::lazy_static;
use mux::Mux;
use mux::pane::CachePolicy;
use std::rc::Rc;
use std::sync::Mutex;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

lazy_static! {
    static ref TOP_BAR_BUTTONS: Mutex<Vec<(TopBarItem, String)>> = {
        let labels = w111erd_config::w111erd_top_bar_buttons();
        if labels.is_empty() {
            Mutex::new(vec![
                (TopBarItem::NewTabButton, "New Tab".to_string()),
                (TopBarItem::SplitHorizontal, "Split H".to_string()),
                (TopBarItem::SplitVertical, "Split V".to_string()),
                (TopBarItem::SplitAuto, "Auto".to_string()),
                (TopBarItem::Launcher, "Launcher".to_string()),
                (TopBarItem::QuickSelect, "Quick Select".to_string()),
            ])
        } else {
            Mutex::new(labels.into_iter().take(6).enumerate()
                .filter_map(|(i, label)| {
                    let item = match i {
                        0 => TopBarItem::NewTabButton,
                        1 => TopBarItem::SplitHorizontal,
                        2 => TopBarItem::SplitVertical,
                        3 => TopBarItem::SplitAuto,
                        4 => TopBarItem::Launcher,
                        5 => TopBarItem::QuickSelect,
                        _ => return None,
                    };
                    Some((item, label))
                })
                .collect())
        }
    };
}

fn blend_color(left: LinearRgba, right: LinearRgba, amount: f32) -> LinearRgba {
    let mix = amount.clamp(0.0, 1.0);
    LinearRgba(
        left.0 + ((right.0 - left.0) * mix),
        left.1 + ((right.1 - left.1) * mix),
        left.2 + ((right.2 - left.2) * mix),
        left.3 + ((right.3 - left.3) * mix),
    )
}

fn rounded_button_corners() -> Corners {
    let corner = SizedPoly {
        width: Dimension::Cells(0.45),
        height: Dimension::Cells(0.45),
        poly: TOP_LEFT_ROUNDED_CORNER,
    };

    Corners {
        top_left: corner,
        top_right: SizedPoly {
            poly: TOP_RIGHT_ROUNDED_CORNER,
            ..corner
        },
        bottom_left: SizedPoly {
            poly: BOTTOM_LEFT_ROUNDED_CORNER,
            ..corner
        },
        bottom_right: SizedPoly {
            poly: BOTTOM_RIGHT_ROUNDED_CORNER,
            ..corner
        },
    }
}

fn top_bar_button(
    font: &Rc<LoadedFont>,
    label: &str,
    item: TopBarItem,
    colors: ElementColors,
    hover_colors: ElementColors,
    left_margin_cells: f32,
) -> Element {
    Element::new(font, ElementContent::Text(label.to_string()))
        .vertical_align(VerticalAlign::Middle)
        .item_type(crate::termwindow::UIItemType::TopBar(item))
        .margin(BoxDimension {
            left: Dimension::Cells(left_margin_cells),
            right: Dimension::Cells(0.0),
            top: Dimension::Cells(0.0),
            bottom: Dimension::Cells(0.0),
        })
        .padding(BoxDimension {
            left: Dimension::Cells(0.45),
            right: Dimension::Cells(0.45),
            top: Dimension::Cells(0.08),
            bottom: Dimension::Cells(0.08),
        })
        .border(BoxDimension::new(Dimension::Pixels(1.0)))
        .border_corners(Some(rounded_button_corners()))
        .colors(colors)
        .hover_colors(Some(hover_colors))
}

fn context_pill(
    font: &Rc<LoadedFont>,
    label: String,
    colors: ElementColors,
    left_margin_cells: f32,
    max_width: Option<Dimension>,
) -> Element {
    Element::new(font, ElementContent::Text(label))
        .display(DisplayType::Inline)
        .vertical_align(VerticalAlign::Middle)
        .margin(BoxDimension {
            left: Dimension::Cells(left_margin_cells),
            right: Dimension::Cells(0.0),
            top: Dimension::Cells(0.0),
            bottom: Dimension::Cells(0.0),
        })
        .padding(BoxDimension {
            left: Dimension::Cells(0.45),
            right: Dimension::Cells(0.45),
            top: Dimension::Cells(0.08),
            bottom: Dimension::Cells(0.08),
        })
        .border(BoxDimension::new(Dimension::Pixels(1.0)))
        .border_corners(Some(rounded_button_corners()))
        .colors(colors)
        .max_width(max_width)
}

fn trim_label(label: &str, max_chars: usize) -> String {
    let char_count = label.chars().count();
    if char_count <= max_chars {
        return label.to_string();
    }

    let trimmed: String = label.chars().take(max_chars.saturating_sub(3)).collect();
    format!("{}...", trimmed)
}

impl crate::TermWindow {
    pub fn paint_top_bar(
        &mut self,
        _layers: &mut TripleLayerQuadAllocator,
    ) -> anyhow::Result<()> {
        let top_bar_height = self.top_bar_pixel_height();
        if top_bar_height <= 0.0 {
            return Ok(());
        }

        let top_bar_y = self.top_bar_pixel_y();
        let palette = self.palette().clone();
        let title_font = self.fonts.title_font()?;
        let metrics = crate::utilsprites::RenderMetrics::with_font_metrics(&title_font.metrics());
        let colors = self
            .config
            .colors
            .as_ref()
            .and_then(|configured| configured.tab_bar.as_ref())
            .cloned()
            .unwrap_or_else(TabBarColors::default);
        let frame_bg = if self.focused.is_some() {
            self.config.window_frame.active_titlebar_bg.to_linear()
        } else {
            self.config.window_frame.inactive_titlebar_bg.to_linear()
        };
        let frame_fg = if self.focused.is_some() {
            self.config.window_frame.active_titlebar_fg.to_linear()
        } else {
            self.config.window_frame.inactive_titlebar_fg.to_linear()
        };
        let strip_bg = blend_color(frame_bg, palette.background.to_linear(), 0.35);
        let strip_border = blend_color(strip_bg, frame_fg, 0.22).mul_alpha(0.9);
        let action_bg = blend_color(strip_bg, colors.inactive_tab().bg_color.to_linear(), 0.55);
        let action_hover_bg = blend_color(
            strip_bg,
            colors.inactive_tab_hover().bg_color.to_linear(),
            0.72,
        );
        let context_bg = blend_color(strip_bg, colors.active_tab().bg_color.to_linear(), 0.42);
        let context_border = blend_color(context_bg, frame_fg, 0.16).mul_alpha(0.9);
        let workspace_bg = blend_color(strip_bg, colors.new_tab().bg_color.to_linear(), 0.35);
        let workspace_border = blend_color(workspace_bg, frame_fg, 0.18).mul_alpha(0.9);
        let action_border = blend_color(action_bg, frame_fg, 0.18);
        let action_hover_border = blend_color(action_hover_bg, frame_fg, 0.28);
        let border = self.get_os_border();
        let width = self.dimensions.pixel_width as f32 - (border.left + border.right).get() as f32;

        let action_colors = ElementColors {
            border: BorderColor::new(action_border),
            bg: action_bg.into(),
            text: frame_fg.into(),
        };
        let action_hover_colors = ElementColors {
            border: BorderColor::new(action_hover_border),
            bg: action_hover_bg.into(),
            text: frame_fg.into(),
        };
        let context_colors = ElementColors {
            border: BorderColor::new(context_border),
            bg: context_bg.into(),
            text: frame_fg.into(),
        };
        let workspace_colors = ElementColors {
            border: BorderColor::new(workspace_border),
            bg: workspace_bg.into(),
            text: frame_fg.into(),
        };

        let mut children = vec![];

        for (idx, (item, label)) in TOP_BAR_BUTTONS.lock().unwrap().iter().enumerate() {
            children.push(top_bar_button(
                &title_font,
                label,
                *item,
                action_colors.clone(),
                action_hover_colors.clone(),
                if idx == 0 { 0.45 } else { 0.18 },
            ));
        }

        let workspace_name = Mux::get()
            .get_window(self.mux_window_id)
            .map(|window| trim_label(window.get_workspace(), 18))
            .unwrap_or_else(|| "default".to_string());
        let active_location = self
            .get_active_pane_or_overlay()
            .and_then(|pane| pane.get_current_working_dir(CachePolicy::AllowStale))
            .map(|url| {
                if let Ok(path) = url.to_file_path() {
                    path.display().to_string()
                } else {
                    url.path().to_string()
                }
            })
            .filter(|path| !path.is_empty())
            .unwrap_or_else(|| "Location unavailable".to_string());

        children.push(context_pill(
            &title_font,
            format!("Workspace {}", workspace_name),
            workspace_colors,
            0.42,
            Some(Dimension::Cells(14.0)),
        ));
        children.push(context_pill(
            &title_font,
            trim_label(&active_location, 42),
            context_colors,
            0.18,
            Some(Dimension::Pixels((width * 0.38).clamp(180.0, 520.0))),
        ));

        let top_bar = Element::new(&title_font, ElementContent::Children(children))
            .display(DisplayType::Block)
            .min_width(Some(Dimension::Pixels(width)))
            .min_height(Some(Dimension::Pixels(top_bar_height)))
            .colors(ElementColors {
                border: BorderColor {
                    left: LinearRgba::TRANSPARENT,
                    top: LinearRgba::TRANSPARENT,
                    right: LinearRgba::TRANSPARENT,
                    bottom: strip_border,
                },
                bg: strip_bg.into(),
                text: frame_fg.into(),
            })
            .border(BoxDimension {
                left: Dimension::Pixels(0.0),
                top: Dimension::Pixels(0.0),
                right: Dimension::Pixels(0.0),
                bottom: Dimension::Pixels(1.0),
            });

        let gl_state = self.render_state.as_ref().unwrap();
        let computed = self.compute_element(
            &LayoutContext {
                height: config::DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: config::DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_width as f32,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(border.left.get() as f32, top_bar_y, width, top_bar_height),
                metrics: &metrics,
                gl_state,
                zindex: 10,
            },
            &top_bar,
        )?;

        let ui_items: Vec<UIItem> = computed.ui_items();
        self.render_element(&computed, gl_state, None)?;
        self.ui_items.extend(ui_items);

        Ok(())
    }
}
