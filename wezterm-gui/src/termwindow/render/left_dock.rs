use crate::quad::TripleLayerQuadAllocator;
use crate::tabbar::{preset_button_count, preset_button_label};
use crate::termwindow::box_model::{
    BorderColor, BoxDimension, Corners, DisplayType, Element, ElementColors, ElementContent,
    LayoutContext, SizedPoly,
};
use crate::termwindow::render::corners::{
    BOTTOM_LEFT_ROUNDED_CORNER, BOTTOM_RIGHT_ROUNDED_CORNER, TOP_LEFT_ROUNDED_CORNER,
    TOP_RIGHT_ROUNDED_CORNER,
};
use crate::termwindow::{LeftDockItem, UIItem, UIItemType};
use config::{Dimension, TabBarColors};
use std::rc::Rc;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

fn blend_color(left: LinearRgba, right: LinearRgba, amount: f32) -> LinearRgba {
    let mix = amount.clamp(0.0, 1.0);
    LinearRgba(
        left.0 + ((right.0 - left.0) * mix),
        left.1 + ((right.1 - left.1) * mix),
        left.2 + ((right.2 - left.2) * mix),
        left.3 + ((right.3 - left.3) * mix),
    )
}

fn rounded_corners() -> Corners {
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

fn dock_button(
    font: &Rc<LoadedFont>,
    label: &str,
    item: LeftDockItem,
    width: f32,
    colors: ElementColors,
    hover_colors: ElementColors,
) -> Element {
    Element::new(font, ElementContent::Text(label.to_string()))
        .display(DisplayType::Block)
        .item_type(UIItemType::LeftDock(item))
        .min_width(Some(Dimension::Pixels(width)))
        .margin(BoxDimension {
            left: Dimension::Cells(0.0),
            right: Dimension::Cells(0.0),
            top: Dimension::Cells(0.24),
            bottom: Dimension::Cells(0.0),
        })
        .padding(BoxDimension {
            left: Dimension::Cells(0.45),
            right: Dimension::Cells(0.45),
            top: Dimension::Cells(0.16),
            bottom: Dimension::Cells(0.16),
        })
        .border(BoxDimension::new(Dimension::Pixels(1.0)))
        .border_corners(Some(rounded_corners()))
        .colors(colors)
        .hover_colors(Some(hover_colors))
}

impl crate::TermWindow {
    pub fn paint_left_dock(
        &mut self,
        _layers: &mut TripleLayerQuadAllocator,
    ) -> anyhow::Result<()> {
        let dock_width = self.left_dock_pixel_width();
        if dock_width <= 0.0 {
            return Ok(());
        }

        let palette = self.palette().clone();
        let title_font = self.fonts.title_font()?;
        let metrics = crate::utilsprites::RenderMetrics::with_font_metrics(&title_font.metrics());
        let border = self.get_os_border();
        let top = self.top_chrome_pixel_height() + border.top.get() as f32;
        let height = self.dimensions.pixel_height as f32
            - top
            - border.bottom.get() as f32
            - self.bottom_chrome_pixel_height();

        if height <= 0.0 {
            return Ok(());
        }

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
        let dock_bg = LinearRgba(0., 0., 0., 1.);
        let dock_border = blend_color(dock_bg, frame_fg, 0.24).mul_alpha(0.92);
        let preset_colors = ElementColors {
            border: BorderColor::new(colors.new_tab().bg_color.to_linear()),
            bg: LinearRgba(0., 0., 0., 1.).into(),
            text: LinearRgba(187./255., 187./255., 187./255., 1.).into(),
        };
        let preset_hover_colors = ElementColors {
            border: BorderColor::new(colors.new_tab_hover().bg_color.to_linear()),
            bg: LinearRgba(51./255., 51./255., 51./255., 1.).into(),
            text: LinearRgba(187./255., 187./255., 187./255., 1.).into(),
        };
        let header_badge_bg = LinearRgba(0., 0., 0., 1.).into();
        let header_badge_border = blend_color(header_badge_bg, frame_fg, 0.24).mul_alpha(0.94);
        let header_text = LinearRgba(187./255., 187./255., 187./255., 1.);
        let header_badge_colors = ElementColors {
            border: BorderColor::new(header_badge_border),
            bg: header_badge_bg.into(),
            text: header_text.into(),
        };

        let button_width = dock_width - (metrics.cell_size.width as f32 * 2.2);
        let mut children = vec![];

        children.push(
            Element::new(&title_font, ElementContent::Text("Presets".to_string()))
                .display(DisplayType::Block)
                .margin(BoxDimension {
                    left: Dimension::Cells(0.0),
                    right: Dimension::Cells(0.0),
                    top: Dimension::Cells(0.12),
                    bottom: Dimension::Cells(0.18),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.35),
                    right: Dimension::Cells(0.35),
                    top: Dimension::Cells(0.12),
                    bottom: Dimension::Cells(0.12),
                })
                .border(BoxDimension::new(Dimension::Pixels(1.0)))
                .border_corners(Some(rounded_corners()))
                .colors(header_badge_colors),
        );

        for idx in 0..preset_button_count() {
            if let Some(label) = preset_button_label(idx) {
                children.push(dock_button(
                    &title_font,
                    &label,
                    LeftDockItem::PresetButton(idx),
                    button_width.max(metrics.cell_size.width as f32 * 4.0),
                    preset_colors.clone(),
                    preset_hover_colors.clone(),
                ));
            }
        }

        let dock = Element::new(&title_font, ElementContent::Children(children))
            .display(DisplayType::Block)
            .min_width(Some(Dimension::Pixels(dock_width)))
            .min_height(Some(Dimension::Pixels(height)))
            .padding(BoxDimension {
                left: Dimension::Cells(0.55),
                right: Dimension::Cells(0.55),
                top: Dimension::Cells(0.35),
                bottom: Dimension::Cells(0.35),
            })
            .border(BoxDimension {
                left: Dimension::Pixels(0.0),
                top: Dimension::Pixels(0.0),
                right: Dimension::Pixels(1.0),
                bottom: Dimension::Pixels(0.0),
            })
            .colors(ElementColors {
                border: BorderColor {
                    left: LinearRgba::TRANSPARENT,
                    top: LinearRgba::TRANSPARENT,
                    right: dock_border,
                    bottom: LinearRgba::TRANSPARENT,
                },
                bg: dock_bg.into(),
                text: frame_fg.into(),
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
                bounds: euclid::rect(border.left.get() as f32, top, dock_width, height),
                metrics: &metrics,
                gl_state,
                zindex: 9,
            },
            &dock,
        )?;

        let ui_items: Vec<UIItem> = computed.ui_items();
        self.render_element(&computed, gl_state, None)?;
        self.ui_items.extend(ui_items);

        Ok(())
    }
}