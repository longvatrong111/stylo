/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// TODO(gw): This contains helper traits and implementations for converting Servo display lists
//           into WebRender display lists. In the future, this step should be completely removed.
//           This might be achieved by sharing types between WR and Servo display lists, or
//           completely converting layout to directly generate WebRender display lists, for example.

use app_units::Au;
use euclid::{Point2D, Rect, Size2D};
use gfx::display_list::{BorderRadii, BoxShadowClipMode, ClippingRegion};
use gfx::display_list::{DisplayItem, DisplayList, DisplayListTraversal, StackingContextType};
use gfx_traits::{FragmentType, ScrollRootId};
use msg::constellation_msg::PipelineId;
use style::computed_values::{image_rendering, mix_blend_mode};
use style::computed_values::filter::{self, Filter};
use style::values::computed::BorderStyle;
use webrender_traits::{self, DisplayListBuilder, ExtendMode, LayoutTransform};

pub trait WebRenderDisplayListConverter {
    fn convert_to_webrender(&self, pipeline_id: PipelineId) -> DisplayListBuilder;
}

trait WebRenderDisplayItemConverter {
    fn convert_to_webrender(&self, builder: &mut DisplayListBuilder);
}

trait ToBorderStyle {
    fn to_border_style(&self) -> webrender_traits::BorderStyle;
}

impl ToBorderStyle for BorderStyle {
    fn to_border_style(&self) -> webrender_traits::BorderStyle {
        match *self {
            BorderStyle::none => webrender_traits::BorderStyle::None,
            BorderStyle::solid => webrender_traits::BorderStyle::Solid,
            BorderStyle::double => webrender_traits::BorderStyle::Double,
            BorderStyle::dotted => webrender_traits::BorderStyle::Dotted,
            BorderStyle::dashed => webrender_traits::BorderStyle::Dashed,
            BorderStyle::hidden => webrender_traits::BorderStyle::Hidden,
            BorderStyle::groove => webrender_traits::BorderStyle::Groove,
            BorderStyle::ridge => webrender_traits::BorderStyle::Ridge,
            BorderStyle::inset => webrender_traits::BorderStyle::Inset,
            BorderStyle::outset => webrender_traits::BorderStyle::Outset,
        }
    }
}
trait ToBoxShadowClipMode {
    fn to_clip_mode(&self) -> webrender_traits::BoxShadowClipMode;
}

impl ToBoxShadowClipMode for BoxShadowClipMode {
    fn to_clip_mode(&self) -> webrender_traits::BoxShadowClipMode {
        match *self {
            BoxShadowClipMode::None => webrender_traits::BoxShadowClipMode::None,
            BoxShadowClipMode::Inset => webrender_traits::BoxShadowClipMode::Inset,
            BoxShadowClipMode::Outset => webrender_traits::BoxShadowClipMode::Outset,
        }
    }
}

trait ToSizeF {
    fn to_sizef(&self) -> webrender_traits::LayoutSize;
}

trait ToPointF {
    fn to_pointf(&self) -> webrender_traits::LayoutPoint;
}

impl ToPointF for Point2D<Au> {
    fn to_pointf(&self) -> webrender_traits::LayoutPoint {
        webrender_traits::LayoutPoint::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

impl ToSizeF for Size2D<Au> {
    fn to_sizef(&self) -> webrender_traits::LayoutSize {
        webrender_traits::LayoutSize::new(self.width.to_f32_px(), self.height.to_f32_px())
    }
}

trait ToRectF {
    fn to_rectf(&self) -> webrender_traits::LayoutRect;
}

impl ToRectF for Rect<Au> {
    fn to_rectf(&self) -> webrender_traits::LayoutRect {
        let x = self.origin.x.to_f32_px();
        let y = self.origin.y.to_f32_px();
        let w = self.size.width.to_f32_px();
        let h = self.size.height.to_f32_px();
        let point = webrender_traits::LayoutPoint::new(x, y);
        let size = webrender_traits::LayoutSize::new(w, h);
        webrender_traits::LayoutRect::new(point, size)
    }
}

trait ToClipRegion {
    fn to_clip_region(&self, builder: &mut DisplayListBuilder) -> webrender_traits::ClipRegion;
}

impl ToClipRegion for ClippingRegion {
    fn to_clip_region(&self, builder: &mut DisplayListBuilder) -> webrender_traits::ClipRegion {
        builder.new_clip_region(&self.main.to_rectf(),
                                self.complex.iter().map(|complex_clipping_region| {
                                    webrender_traits::ComplexClipRegion::new(
                                        complex_clipping_region.rect.to_rectf(),
                                        complex_clipping_region.radii.to_border_radius(),
                                     )
                                }).collect(),
                                None)
    }
}

trait ToBorderRadius {
    fn to_border_radius(&self) -> webrender_traits::BorderRadius;
}

impl ToBorderRadius for BorderRadii<Au> {
    fn to_border_radius(&self) -> webrender_traits::BorderRadius {
        webrender_traits::BorderRadius {
            top_left: self.top_left.to_sizef(),
            top_right: self.top_right.to_sizef(),
            bottom_left: self.bottom_left.to_sizef(),
            bottom_right: self.bottom_right.to_sizef(),
        }
    }
}

trait ToBlendMode {
    fn to_blend_mode(&self) -> webrender_traits::MixBlendMode;
}

impl ToBlendMode for mix_blend_mode::T {
    fn to_blend_mode(&self) -> webrender_traits::MixBlendMode {
        match *self {
            mix_blend_mode::T::normal => webrender_traits::MixBlendMode::Normal,
            mix_blend_mode::T::multiply => webrender_traits::MixBlendMode::Multiply,
            mix_blend_mode::T::screen => webrender_traits::MixBlendMode::Screen,
            mix_blend_mode::T::overlay => webrender_traits::MixBlendMode::Overlay,
            mix_blend_mode::T::darken => webrender_traits::MixBlendMode::Darken,
            mix_blend_mode::T::lighten => webrender_traits::MixBlendMode::Lighten,
            mix_blend_mode::T::color_dodge => webrender_traits::MixBlendMode::ColorDodge,
            mix_blend_mode::T::color_burn => webrender_traits::MixBlendMode::ColorBurn,
            mix_blend_mode::T::hard_light => webrender_traits::MixBlendMode::HardLight,
            mix_blend_mode::T::soft_light => webrender_traits::MixBlendMode::SoftLight,
            mix_blend_mode::T::difference => webrender_traits::MixBlendMode::Difference,
            mix_blend_mode::T::exclusion => webrender_traits::MixBlendMode::Exclusion,
            mix_blend_mode::T::hue => webrender_traits::MixBlendMode::Hue,
            mix_blend_mode::T::saturation => webrender_traits::MixBlendMode::Saturation,
            mix_blend_mode::T::color => webrender_traits::MixBlendMode::Color,
            mix_blend_mode::T::luminosity => webrender_traits::MixBlendMode::Luminosity,
        }
    }
}

trait ToImageRendering {
    fn to_image_rendering(&self) -> webrender_traits::ImageRendering;
}

impl ToImageRendering for image_rendering::T {
    fn to_image_rendering(&self) -> webrender_traits::ImageRendering {
        match *self {
            image_rendering::T::crisp_edges => webrender_traits::ImageRendering::CrispEdges,
            image_rendering::T::auto => webrender_traits::ImageRendering::Auto,
            image_rendering::T::pixelated => webrender_traits::ImageRendering::Pixelated,
        }
    }
}

trait ToFilterOps {
    fn to_filter_ops(&self) -> Vec<webrender_traits::FilterOp>;
}

impl ToFilterOps for filter::T {
    fn to_filter_ops(&self) -> Vec<webrender_traits::FilterOp> {
        let mut result = Vec::with_capacity(self.filters.len());
        for filter in self.filters.iter() {
            match *filter {
                Filter::Blur(radius) => result.push(webrender_traits::FilterOp::Blur(radius)),
                Filter::Brightness(amount) => result.push(webrender_traits::FilterOp::Brightness(amount)),
                Filter::Contrast(amount) => result.push(webrender_traits::FilterOp::Contrast(amount)),
                Filter::Grayscale(amount) => result.push(webrender_traits::FilterOp::Grayscale(amount)),
                Filter::HueRotate(angle) => result.push(webrender_traits::FilterOp::HueRotate(angle.0)),
                Filter::Invert(amount) => result.push(webrender_traits::FilterOp::Invert(amount)),
                Filter::Opacity(amount) => result.push(webrender_traits::FilterOp::Opacity(amount.into())),
                Filter::Saturate(amount) => result.push(webrender_traits::FilterOp::Saturate(amount)),
                Filter::Sepia(amount) => result.push(webrender_traits::FilterOp::Sepia(amount)),
            }
        }
        result
    }
}

impl WebRenderDisplayListConverter for DisplayList {
    fn convert_to_webrender(&self, pipeline_id: PipelineId) -> DisplayListBuilder {
        let traversal = DisplayListTraversal::new(self);
        let mut builder = DisplayListBuilder::new(pipeline_id.to_webrender());
        for item in traversal {
            item.convert_to_webrender(&mut builder);
        }
        builder
    }
}

impl WebRenderDisplayItemConverter for DisplayItem {
    fn convert_to_webrender(&self, builder: &mut DisplayListBuilder) {
        match *self {
            DisplayItem::SolidColor(ref item) => {
                let color = item.color;
                if color.a > 0.0 {
                    let clip = item.base.clip.to_clip_region(builder);
                    builder.push_rect(item.base.bounds.to_rectf(), clip, color);
                }
            }
            DisplayItem::Text(ref item) => {
                let mut origin = item.baseline_origin.clone();
                let mut glyphs = vec!();

                for slice in item.text_run.natural_word_slices_in_visual_order(&item.range) {
                    for glyph in slice.glyphs.iter_glyphs_for_byte_range(&slice.range) {
                        let glyph_advance = if glyph.char_is_space() {
                            glyph.advance() + item.text_run.extra_word_spacing
                        } else {
                            glyph.advance()
                        };
                        if !slice.glyphs.is_whitespace() {
                            let glyph_offset = glyph.offset().unwrap_or(Point2D::zero());
                            let glyph = webrender_traits::GlyphInstance {
                                index: glyph.id(),
                                point: Point2D::new((origin.x + glyph_offset.x).to_f32_px(),
                                                    (origin.y + glyph_offset.y).to_f32_px()),
                            };
                            glyphs.push(glyph);
                        }
                        origin.x = origin.x + glyph_advance;
                    };
                }

                if glyphs.len() > 0 {
                    let clip = item.base.clip.to_clip_region(builder);
                    builder.push_text(item.base.bounds.to_rectf(),
                                      clip,
                                      glyphs,
                                      item.text_run.font_key,
                                      item.text_color,
                                      item.text_run.actual_pt_size,
                                      item.blur_radius,
                                      None);
                }
            }
            DisplayItem::Image(ref item) => {
                if let Some(id) = item.webrender_image.key {
                    if item.stretch_size.width > Au(0) &&
                       item.stretch_size.height > Au(0) {
                        let clip = item.base.clip.to_clip_region(builder);
                        builder.push_image(item.base.bounds.to_rectf(),
                                           clip,
                                           item.stretch_size.to_sizef(),
                                           item.tile_spacing.to_sizef(),
                                           item.image_rendering.to_image_rendering(),
                                           id);
                    }
                }
            }
            DisplayItem::WebGL(ref item) => {
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_webgl_canvas(item.base.bounds.to_rectf(), clip, item.context_id);
            }
            DisplayItem::Border(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let left = webrender_traits::BorderSide {
                    width: item.border_widths.left.to_f32_px(),
                    color: item.color.left,
                    style: item.style.left.to_border_style(),
                };
                let top = webrender_traits::BorderSide {
                    width: item.border_widths.top.to_f32_px(),
                    color: item.color.top,
                    style: item.style.top.to_border_style(),
                };
                let right = webrender_traits::BorderSide {
                    width: item.border_widths.right.to_f32_px(),
                    color: item.color.right,
                    style: item.style.right.to_border_style(),
                };
                let bottom = webrender_traits::BorderSide {
                    width: item.border_widths.bottom.to_f32_px(),
                    color: item.color.bottom,
                    style: item.style.bottom.to_border_style(),
                };
                let radius = item.radius.to_border_radius();
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_border(rect,
                                    clip,
                                    left,
                                    top,
                                    right,
                                    bottom,
                                    radius);
            }
            DisplayItem::Gradient(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let start_point = item.start_point.to_pointf();
                let end_point = item.end_point.to_pointf();
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_gradient(rect,
                                      clip,
                                      start_point,
                                      end_point,
                                      item.stops.clone(),
                                      ExtendMode::Clamp);
            }
            DisplayItem::Line(..) => {
                println!("TODO DisplayItem::Line");
            }
            DisplayItem::BoxShadow(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let box_bounds = item.box_bounds.to_rectf();
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_box_shadow(rect,
                                        clip,
                                        box_bounds,
                                        item.offset.to_pointf(),
                                        item.color,
                                        item.blur_radius.to_f32_px(),
                                        item.spread_radius.to_f32_px(),
                                        item.border_radius.to_f32_px(),
                                        item.clip_mode.to_clip_mode());
            }
            DisplayItem::Iframe(ref item) => {
                let rect = item.base.bounds.to_rectf();
                let pipeline_id = item.iframe.to_webrender();
                let clip = item.base.clip.to_clip_region(builder);
                builder.push_iframe(rect, clip, pipeline_id);
            }
            DisplayItem::PushStackingContext(ref item) => {
                let stacking_context = &item.stacking_context;
                debug_assert!(stacking_context.context_type == StackingContextType::Real);

                let clip = builder.new_clip_region(&stacking_context.overflow.to_rectf(),
                                                   vec![],
                                                   None);

                builder.push_stacking_context(stacking_context.scroll_policy,
                                              stacking_context.bounds.to_rectf(),
                                              clip,
                                              stacking_context.z_index,
                                              LayoutTransform::from_untyped(&stacking_context.transform).into(),
                                              LayoutTransform::from_untyped(&stacking_context.perspective),
                                              stacking_context.blend_mode.to_blend_mode(),
                                              stacking_context.filters.to_filter_ops());
            }
            DisplayItem::PopStackingContext(_) => builder.pop_stacking_context(),
            DisplayItem::PushScrollRoot(ref item) => {
                let clip = builder.new_clip_region(&item.scroll_root.clip.to_rectf(),
                                                   vec![],
                                                   None);

                builder.push_scroll_layer(clip,
                                          item.scroll_root.size.to_sizef(),
                                          item.scroll_root.id.convert_to_webrender());
            }
            DisplayItem::PopScrollRoot(_) => builder.pop_scroll_layer(),
        }
    }
}

trait WebRenderScrollRootIdConverter {
    fn convert_to_webrender(&self) -> webrender_traits::ServoScrollRootId;
}

impl WebRenderScrollRootIdConverter for ScrollRootId {
    fn convert_to_webrender(&self) -> webrender_traits::ServoScrollRootId {
        webrender_traits::ServoScrollRootId(self.0)
    }
}

trait WebRenderFragmentTypeConverter {
    fn convert_to_webrender(&self) -> webrender_traits::FragmentType;
}

impl WebRenderFragmentTypeConverter for FragmentType {
    fn convert_to_webrender(&self) -> webrender_traits::FragmentType {
        match *self {
            FragmentType::FragmentBody => webrender_traits::FragmentType::FragmentBody,
            FragmentType::BeforePseudoContent => {
                webrender_traits::FragmentType::BeforePseudoContent
            }
            FragmentType::AfterPseudoContent => webrender_traits::FragmentType::AfterPseudoContent,
        }
    }
}
