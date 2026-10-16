use gpui::{Bounds, Hsla, IntoElement, PathBuilder, Pixels, Styled, Window, point, px};
use std::cell::RefCell;
use std::rc::Rc;

pub(super) const CANVAS_INSET: f32 = 2.0;
pub(super) const CANVAS_RADIUS: f32 = 8.0;
pub(super) const INPUT_ISLAND_INSET: f32 = 10.0;
const INPUT_ISLAND_MAX_WIDTH: f32 = 960.0;

/// The island and its popovers share one horizontal geometry, including on
/// wide windows where the composer stops growing but the terminal does not.
pub(super) fn input_island_geometry(width: f32, depth: f32) -> (f32, f32) {
    let inset = (CANVAS_INSET + INPUT_ISLAND_INSET) * depth;
    let available = (width - 2.0 * inset).max(0.0);
    let max_width = width + (INPUT_ISLAND_MAX_WIDTH - width) * depth;
    let island_width = available.min(max_width.max(0.0));
    ((width - island_width) * 0.5, island_width)
}

pub(super) fn input_safe_height(height: f32, progress: f32, depth: f32) -> f32 {
    (height + 2.0 * INPUT_ISLAND_INSET * depth) * progress.clamp(0.0, 1.0)
}

pub(super) fn caption_width(window: f32, sidebar: f32, dock: f32, leading: f32) -> f32 {
    if dock > 0.0 && sidebar >= caption_minimum_width(leading) {
        sidebar.min(window)
    } else {
        window
    }
}

pub(super) fn caption_minimum_width(leading: f32) -> f32 {
    leading + 3.0 * 22.0 + 24.0
}

pub(super) fn input_reveal_clip(height: f32, progress: f32, depth: f32) -> gpui::Div {
    gpui::div()
        .absolute()
        .bottom_0()
        .left_0()
        .w_full()
        .h(px(input_safe_height(height, progress, depth)))
        .overflow_hidden()
}

#[derive(Default)]
pub(super) struct CanvasSurround {
    cache: Rc<RefCell<Option<SurroundGeometry>>>,
}

struct SurroundGeometry {
    bounds: Bounds<Pixels>,
    depth: f32,
    scale: f32,
    corners: Vec<gpui::Path<Pixels>>,
}

impl CanvasSurround {
    pub fn render(
        &self,
        color: Hsla,
        depth: f32,
        safe_height: f32,
        glass: Hsla,
    ) -> impl IntoElement + use<> {
        let cache = self.cache.clone();
        gpui::canvas(
            move |bounds, window, _| {
                let scale = window.scale_factor();
                let bounds = crate::terminal_geometry::snap_terminal_bounds(bounds, scale);
                let inset = canvas_inset(bounds, depth);
                let inner = bounds.inset(inset);
                prepare_native_canvas(inner, CANVAS_RADIUS * depth, window);
                prepare_native_safe_area(safe_height, glass, window);
                let mut cache = cache.borrow_mut();
                if cache.as_ref().is_none_or(|old| {
                    old.bounds != bounds || old.depth != depth || old.scale != scale
                }) {
                    *cache = Some(SurroundGeometry {
                        bounds,
                        depth,
                        scale,
                        corners: corner_cutouts(inner, CANVAS_RADIUS * depth, scale),
                    });
                }
                (
                    surround_strips(bounds, inset),
                    cache.as_ref().unwrap().corners.clone(),
                )
            },
            move |_, (strips, corners), window, _| {
                for strip in strips {
                    if strip.size.width > px(0.0) && strip.size.height > px(0.0) {
                        window.paint_quad(gpui::fill(strip, color));
                    }
                }
                for corner in corners {
                    window.paint_path(corner, color);
                }
            },
        )
        .absolute()
        .size_full()
    }
}

fn corner_cutouts(bounds: Bounds<Pixels>, radius: f32, scale: f32) -> Vec<gpui::Path<Pixels>> {
    let r = px(radius)
        .min(bounds.size.width * 0.5)
        .min(bounds.size.height * 0.5);
    if r <= px(0.0) {
        return Vec::new();
    }
    // Independent AA masks are not complementary under source-over. Extend
    // only the corner cutout one physical pixel inward to cover that fringe.
    let band = px(1.0 / scale.max(1.0)).min(r);
    let c = (r - band) * 0.552_284_8;
    let mut corners = Vec::with_capacity(4);
    for (origin, x_sign, y_sign) in [
        (bounds.origin, 1.0, 1.0),
        (bounds.top_right(), -1.0, 1.0),
        (bounds.bottom_right(), -1.0, -1.0),
        (bounds.bottom_left(), 1.0, -1.0),
    ] {
        let at = |x: Pixels, y: Pixels| point(origin.x + x * x_sign, origin.y + y * y_sign);
        let mut path = PathBuilder::fill();
        path.move_to(origin);
        path.line_to(at(r, px(0.0)));
        path.line_to(at(r, band));
        path.cubic_bezier_to(at(band, r), at(r - c, band), at(band, r - c));
        path.line_to(at(px(0.0), r));
        path.close();
        if let Ok(path) = path.build() {
            corners.push(path);
        }
    }
    corners
}

fn canvas_inset(bounds: Bounds<Pixels>, depth: f32) -> Pixels {
    px(CANVAS_INSET * depth)
        .min(bounds.size.width * 0.5)
        .min(bounds.size.height * 0.5)
        .max(px(0.0))
}

fn surround_strips(bounds: Bounds<Pixels>, inset: Pixels) -> [Bounds<Pixels>; 4] {
    let inner = bounds.inset(inset);
    [
        Bounds::new(bounds.origin, gpui::size(bounds.size.width, inset)),
        Bounds::new(
            point(bounds.left(), bounds.bottom() - inset),
            gpui::size(bounds.size.width, inset),
        ),
        Bounds::new(
            point(bounds.left(), inner.top()),
            gpui::size(inset, inner.size.height),
        ),
        Bounds::new(
            point(inner.right(), inner.top()),
            gpui::size(inset, inner.size.height),
        ),
    ]
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CaptionLayout {
    pub dock: f32,
    pub header: f32,
}

impl CaptionLayout {
    // The sidebar's existing spring owns time. Caption geometry must not
    // start another spring after that sidebar finally becomes wide enough.
    pub fn new(dock: f32) -> Self {
        let dock = dock.clamp(0.0, 1.0);
        Self {
            dock,
            header: 1.0 - dock,
        }
    }
}

pub(super) fn prepare_native_canvas(bounds: Bounds<Pixels>, radius: f32, window: &Window) {
    use raw_window_handle::HasWindowHandle;
    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    let raw_window_handle::RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return;
    };
    let bounds = crate::terminal_geometry::snap_terminal_bounds(bounds, window.scale_factor());
    let changed = unsafe {
        con_terminal_canvas_set_clip(
            handle.ns_view.as_ptr(),
            f64::from(bounds.left()),
            f64::from(bounds.top()),
            f64::from(bounds.size.width),
            f64::from(bounds.size.height),
            f64::from(radius),
        )
    };
    if changed {
        window.request_native_surface_presentation_sync();
    }
}

fn prepare_native_safe_area(height: f32, color: Hsla, window: &Window) {
    use raw_window_handle::HasWindowHandle;
    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    let raw_window_handle::RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return;
    };
    let color: gpui::Rgba = color.into();
    if unsafe {
        con_terminal_canvas_set_safe_area(
            handle.ns_view.as_ptr(),
            height as f64,
            color.r as f64,
            color.g as f64,
            color.b as f64,
            color.a as f64,
        )
    } {
        window.request_native_surface_presentation_sync();
    }
}

unsafe extern "C" {
    fn con_terminal_canvas_set_safe_area(
        view: *mut std::ffi::c_void,
        height: f64,
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
    ) -> bool;
    fn con_terminal_canvas_radius(view: *mut std::ffi::c_void) -> f64;
    fn con_terminal_canvas_set_clip(
        view: *mut std::ffi::c_void,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        radius: f64,
    ) -> bool;
    fn con_terminal_canvas_header_corners(
        view: *mut std::ffi::c_void,
        x: f64,
        y: f64,
        width: f64,
    ) -> u8;
}

pub(crate) fn header_background(color: Hsla) -> impl IntoElement {
    gpui::canvas(
        |_, _, _| {},
        move |bounds, (), window, _| {
            use raw_window_handle::HasWindowHandle;
            let (corners, radius) = HasWindowHandle::window_handle(window)
                .ok()
                .and_then(|handle| {
                    if let raw_window_handle::RawWindowHandle::AppKit(handle) = handle.as_raw() {
                        Some(unsafe {
                            (
                                con_terminal_canvas_header_corners(
                                    handle.ns_view.as_ptr(),
                                    f64::from(bounds.left()),
                                    f64::from(bounds.top()),
                                    f64::from(bounds.size.width),
                                ),
                                con_terminal_canvas_radius(handle.ns_view.as_ptr()) as f32,
                            )
                        })
                    } else {
                        None
                    }
                })
                .unwrap_or((0, 0.0));
            let mut quad = gpui::fill(bounds, color);
            quad.corner_radii.top_left = px(if corners & 1 != 0 { radius } else { 0.0 });
            quad.corner_radii.top_right = px(if corners & 2 != 0 { radius } else { 0.0 });
            window.paint_quad(quad);
        },
    )
    .absolute()
    .size_full()
    .top_0()
    .left_0()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_input_stays_centered_and_popovers_follow_its_frame() {
        for width in [0.0, 20.0, 320.0, 900.0, 1600.0] {
            for depth in [0.0, 0.5, 1.0] {
                let (left, island) = input_island_geometry(width, depth);
                assert!(left >= 0.0 && island >= 0.0);
                assert!((left * 2.0 + island - width).abs() < 0.001);
                assert!(left + island <= width);
                if depth == 0.0 {
                    assert_eq!((left, island), (0.0, width));
                }
            }
        }
        assert_eq!(input_island_geometry(1600.0, 1.0), (320.0, 960.0));
        assert_eq!(input_island_geometry(900.0, 1.0), (12.0, 876.0));
        assert_eq!(input_safe_height(44.0, 1.0, 1.0), 64.0);
        assert_eq!(input_safe_height(44.0, 0.0, 1.0), 0.0);
    }

    #[test]
    fn caption_controls_fit_while_navigation_width_changes() {
        for sidebar in [44.0, 90.0, 184.0, 230.0] {
            for dock in [0.0, 0.5, 1.0] {
                let width = caption_width(1000.0, sidebar, dock, 78.0);
                assert!(width >= 168.0);
                assert!(width <= 1000.0);
                if dock > 0.0 && sidebar >= caption_minimum_width(78.0) {
                    assert_eq!(width, sidebar, "toolbar crossed native terminal content");
                } else {
                    assert_eq!(width, 1000.0);
                }
            }
        }
    }

    #[gpui::test]
    fn input_reveal_changes_the_clip_not_the_composer_position(cx: &mut gpui::TestAppContext) {
        use gpui::{
            Context, InteractiveElement, IntoElement, ParentElement, Render, Window, div, size,
        };
        struct Reveal {
            progress: f32,
        }
        impl Render for Reveal {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().relative().w_full().h(px(64.0)).child(
                    input_reveal_clip(44.0, self.progress, 1.0)
                        .debug_selector(|| "input-reveal-clip".into())
                        .child(
                            div()
                                .absolute()
                                .bottom(px(10.0))
                                .left(px(10.0))
                                .w(px(300.0))
                                .h(px(44.0))
                                .debug_selector(|| "fixed-composer".into()),
                        ),
                )
            }
        }
        let (view, cx) = cx.add_window_view(|_, _| Reveal { progress: 1.0 });
        cx.simulate_resize(size(px(500.0), px(300.0)));
        let fixed = cx.debug_bounds("fixed-composer").unwrap();
        for progress in [0.25, 0.7, 0.4, 1.0] {
            view.update(cx, |view, cx| {
                view.progress = progress;
                cx.notify();
            });
            assert_eq!(cx.debug_bounds("fixed-composer").unwrap(), fixed);
            let clip = cx.debug_bounds("input-reveal-clip").unwrap();
            assert_eq!(clip.bottom(), px(64.0));
            assert!(
                (clip.size.height.as_f32() - 64.0 * progress).abs() <= 0.5,
                "clip height must round to the test window's pixel grid"
            );
        }
    }

    #[gpui::test]
    fn native_safe_area_is_cached_passive_and_removed_from_clean_layouts() {
        use cocoa::foundation::{NSPoint, NSRect, NSSize};
        use objc::{class, msg_send, sel, sel_impl};
        unsafe {
            let parent: *mut objc::runtime::Object = msg_send![class!(NSView), alloc];
            let parent: *mut objc::runtime::Object = msg_send![parent, initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(600.0, 400.0))];
            let gpui: *mut objc::runtime::Object = msg_send![class!(NSView), alloc];
            let gpui: *mut objc::runtime::Object = msg_send![gpui, initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(600.0, 400.0))];
            let _: () = msg_send![parent, addSubview: gpui];
            con_terminal_canvas_set_clip(gpui.cast(), 4.0, 4.0, 592.0, 392.0, 8.0);
            assert!(!con_terminal_canvas_set_safe_area(
                gpui.cast(),
                0.0,
                1.0,
                0.9,
                0.8,
                0.9
            ));
            assert!(con_terminal_canvas_set_safe_area(
                gpui.cast(),
                64.0,
                1.0,
                0.9,
                0.8,
                0.9
            ));
            assert!(!con_terminal_canvas_set_safe_area(
                gpui.cast(),
                64.0,
                1.0,
                0.9,
                0.8,
                0.9
            ));
            let children: *mut objc::runtime::Object = msg_send![parent, subviews];
            let count: usize = msg_send![children, count];
            assert_eq!(count, 2);
            let material: *mut objc::runtime::Object = msg_send![children, objectAtIndex: 0usize];
            let rect: NSRect = msg_send![material, frame];
            assert_eq!(
                (
                    rect.origin.x,
                    rect.origin.y,
                    rect.size.width,
                    rect.size.height
                ),
                (4.0, 4.0, 592.0, 64.0)
            );
            let hit: *mut objc::runtime::Object =
                msg_send![material, hitTest: NSPoint::new(20.0, 20.0)];
            assert!(hit.is_null());
            let accepts: bool = msg_send![material, acceptsFirstResponder];
            assert!(!accepts);
            assert!(con_terminal_canvas_set_safe_area(
                gpui.cast(),
                0.0,
                1.0,
                0.9,
                0.8,
                0.9
            ));
            let hidden: bool = msg_send![material, isHidden];
            assert!(hidden);
            assert!(con_terminal_canvas_set_safe_area(
                gpui.cast(),
                64.0,
                0.2,
                0.2,
                0.2,
                0.8
            ));
            let layer: *mut objc::runtime::Object = msg_send![material, layer];
            let mask: *mut objc::runtime::Object = msg_send![layer, mask];
            assert!(!mask.is_null());
            let count: usize = msg_send![children, count];
            assert_eq!(count, 2, "retargeting allocated another native material");
            let _: () = msg_send![gpui, release];
            let _: () = msg_send![parent, release];
        }
    }

    #[test]
    fn chrome_surround_never_paints_the_terminal_interior() {
        let bounds = Bounds::new(point(px(10.0), px(20.0)), gpui::size(px(600.0), px(400.0)));
        let strips = surround_strips(bounds, canvas_inset(bounds, 1.0));
        let contains = |x: f32, y: f32| {
            let p = point(px(x), px(y));
            strips.iter().any(|strip| strip.contains(&p))
        };
        for (x, y) in [(11.0, 220.0), (609.0, 220.0), (310.0, 21.0), (310.0, 419.0)] {
            assert!(contains(x, y), "missing chrome at {x},{y}");
        }
        for (x, y) in [(310.0, 220.0), (26.0, 28.0), (310.0, 28.0), (602.0, 220.0)] {
            assert!(!contains(x, y), "chrome painted terminal at {x},{y}");
        }
        assert_eq!(canvas_inset(bounds, 0.0), px(0.0));
        assert!(
            surround_strips(bounds, px(0.0))
                .iter()
                .all(|strip| strip.size.width == px(0.0) || strip.size.height == px(0.0))
        );
    }

    #[test]
    fn corner_paths_are_small_and_cover_the_native_antialiasing_band() {
        let bounds = Bounds::new(point(px(0.0), px(0.0)), gpui::size(px(600.0), px(400.0)));
        for scale in [1.0, 2.0] {
            let paths = corner_cutouts(bounds, CANVAS_RADIUS, scale);
            assert_eq!(paths.len(), 4);
            for path in &paths {
                assert!(path.bounds.size.width <= px(CANVAS_RADIUS));
                assert!(path.bounds.size.height <= px(CANVAS_RADIUS));
            }
            let contains = |x, y| {
                let p = point(px(x), px(y));
                paths[0].vertices.chunks_exact(3).any(|triangle| {
                    let cross = |a: gpui::Point<Pixels>, b: gpui::Point<Pixels>| {
                        (b.x - a.x).as_f32() * (p.y - a.y).as_f32()
                            - (b.y - a.y).as_f32() * (p.x - a.x).as_f32()
                    };
                    let [a, b, c] = [
                        triangle[0].xy_position,
                        triangle[1].xy_position,
                        triangle[2].xy_position,
                    ];
                    let sides = [cross(a, b), cross(b, c), cross(c, a)];
                    sides.iter().all(|s| *s >= 0.0) || sides.iter().all(|s| *s <= 0.0)
                })
            };
            assert!(contains(0.1, 0.1));
            assert!(contains(2.64, 2.64), "missing AA overlap at {scale}x");
            assert!(!contains(8.0, 8.0), "corner painted terminal interior");
        }
        assert!(corner_cutouts(bounds, 0.0, 2.0).is_empty());
    }

    #[gpui::test]
    fn native_canvas_clip_tracks_parent_geometry_and_reuses_unchanged_masks() {
        use cocoa::foundation::{NSPoint, NSRect, NSSize};
        use objc::{class, msg_send, sel, sel_impl};
        use std::ffi::c_void;
        unsafe extern "C" {
            fn con_terminal_canvas_generation(view: *mut c_void) -> u64;
            fn con_terminal_canvas_apply_clip(view: *mut c_void);
            fn CGPathContainsPoint(
                path: *const c_void,
                transform: *const c_void,
                point: NSPoint,
                even_odd: bool,
            ) -> bool;
        }
        unsafe {
            let parent: *mut objc::runtime::Object = msg_send![class!(NSView), alloc];
            let parent: *mut objc::runtime::Object = msg_send![parent, initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(600.0, 400.0))];
            let host: *mut objc::runtime::Object = msg_send![class!(NSView), alloc];
            let host: *mut objc::runtime::Object = msg_send![host, initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(600.0, 400.0))];
            let _: () = msg_send![parent, addSubview: host];
            let _: () = msg_send![host, setWantsLayer: true];
            let parent_ptr = parent.cast();
            assert!(con_terminal_canvas_set_clip(
                parent_ptr, 6.0, 6.0, 588.0, 388.0, 9.0
            ));
            assert_eq!(con_terminal_canvas_generation(host.cast()), 1);
            let subviews: *mut objc::runtime::Object = msg_send![parent, subviews];
            let view_count: usize = msg_send![subviews, count];
            assert_eq!(view_count, 1, "edge material added an input-receiving view");
            con_terminal_canvas_apply_clip(host.cast());
            let layer: *mut objc::runtime::Object = msg_send![host, layer];
            let mask: *mut objc::runtime::Object = msg_send![layer, mask];
            assert!(!mask.is_null());
            let path: *const c_void = msg_send![mask, path];
            let contains =
                |x, y| CGPathContainsPoint(path, std::ptr::null(), NSPoint::new(x, y), false);
            assert!(contains(300.0, 200.0));
            assert!(contains(15.0, 7.0));
            assert!(!contains(6.5, 6.5));
            assert!(!contains(2.0, 200.0));
            assert!(!con_terminal_canvas_set_clip(
                parent_ptr, 6.0, 6.0, 588.0, 388.0, 9.0
            ));
            con_terminal_canvas_apply_clip(host.cast());
            let same_path: *const c_void = msg_send![mask, path];
            assert_eq!(path, same_path, "idle layout rebuilt the mask");
            assert_eq!(
                con_terminal_canvas_header_corners(parent_ptr, 6.0, 6.0, 588.0),
                3
            );
            assert_eq!(
                con_terminal_canvas_header_corners(parent_ptr, 300.0, 6.0, 294.0),
                2
            );
            assert_eq!(
                con_terminal_canvas_header_corners(parent_ptr, 6.0, 100.0, 588.0),
                0
            );
            assert!(con_terminal_canvas_set_clip(
                parent_ptr, 12.0, 6.0, 580.0, 300.0, 9.0
            ));
            assert_eq!(con_terminal_canvas_generation(host.cast()), 2);
            con_terminal_canvas_apply_clip(host.cast());
            let changed_path: *const c_void = msg_send![mask, path];
            assert_ne!(changed_path, same_path);
            assert!(!CGPathContainsPoint(
                changed_path,
                std::ptr::null(),
                NSPoint::new(15.0, 50.0),
                false
            ));
            assert!(con_terminal_canvas_set_clip(
                parent_ptr, 6.0, 6.0, 588.0, 388.0, 9.0
            ));
            let _: () = msg_send![host, setFrame: NSRect::new(NSPoint::new(298.0, 4.0), NSSize::new(298.0, 392.0))];
            con_terminal_canvas_apply_clip(host.cast());
            let split_path: *const c_void = msg_send![mask, path];
            assert!(CGPathContainsPoint(
                split_path,
                std::ptr::null(),
                NSPoint::new(100.0, 200.0),
                false
            ));
            assert!(!CGPathContainsPoint(
                split_path,
                std::ptr::null(),
                NSPoint::new(295.5, 389.5),
                false
            ));
            let generation = con_terminal_canvas_generation(host.cast());
            assert!(!con_terminal_canvas_set_clip(
                parent_ptr,
                f64::NAN,
                0.0,
                10.0,
                10.0,
                9.0
            ));
            assert_eq!(con_terminal_canvas_generation(host.cast()), generation);
            assert!(con_terminal_canvas_set_clip(
                parent_ptr, 0.0, 0.0, 600.0, 400.0, 0.0
            ));
            con_terminal_canvas_apply_clip(host.cast());
            let clean_mask: *mut objc::runtime::Object = msg_send![layer, mask];
            assert!(
                clean_mask.is_null(),
                "clean terminal retained an offscreen mask"
            );
            assert!(con_terminal_canvas_set_clip(
                parent_ptr, 0.0, 0.0, 1.0, 1.0, 9.0
            ));
            con_terminal_canvas_apply_clip(host.cast());
            assert!(con_terminal_canvas_set_clip(
                parent_ptr, 0.0, 0.0, 0.0, 0.0, 9.0
            ));
            con_terminal_canvas_apply_clip(host.cast());
            assert_eq!(
                con_terminal_canvas_header_corners(parent_ptr, 0.0, 0.0, 0.0),
                0
            );
            let _: () = msg_send![host, release];
            let _: () = msg_send![parent, release];
        }
    }
}
