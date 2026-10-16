use crate::motion::MotionValue;
use gpui::{App, Bounds, Hsla, IntoElement, PathBuilder, Pixels, Styled, Window, point, px};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub(super) const CANVAS_INSET: f32 = 6.0;
pub(super) const CANVAS_RADIUS: f32 = 9.0;
pub(super) const INPUT_ISLAND_INSET: f32 = 8.0;

#[derive(Default)]
pub(super) struct CanvasSurround {
    path: Rc<RefCell<Option<SurroundPath>>>,
}

struct SurroundPath {
    bounds: Bounds<Pixels>,
    geometry: SurroundGeometry,
}

#[derive(Clone)]
struct SurroundGeometry {
    strips: [Bounds<Pixels>; 4],
    corners: Vec<gpui::Path<Pixels>>,
}

impl CanvasSurround {
    pub fn render(&self, color: Hsla) -> impl IntoElement + use<> {
        let cache = self.path.clone();
        gpui::canvas(
            move |bounds, window, _| {
                let bounds =
                    crate::terminal_geometry::snap_terminal_bounds(bounds, window.scale_factor());
                prepare_native_canvas(bounds.inset(px(CANVAS_INSET)), window);
                let mut cache = cache.borrow_mut();
                if cache.as_ref().is_none_or(|cached| cached.bounds != bounds) {
                    *cache = Some(SurroundPath {
                        bounds,
                        geometry: surround_geometry(bounds),
                    });
                }
                cache.as_ref().map(|cached| cached.geometry.clone())
            },
            move |_, path, window, _| {
                if let Some(geometry) = path {
                    for strip in geometry.strips {
                        window.paint_quad(gpui::fill(strip, color));
                    }
                    for corner in geometry.corners {
                        window.paint_path(corner, color);
                    }
                }
            },
        )
        .absolute()
        .size_full()
    }
}

// Keep each path's bounds tiny: GPUI composites its path intermediate texture
// over that rectangle. A window-sized path with a hole still costs a large blit.
fn surround_geometry(bounds: Bounds<Pixels>) -> SurroundGeometry {
    let inset = px(CANVAS_INSET)
        .min(bounds.size.width * 0.5)
        .min(bounds.size.height * 0.5)
        .max(px(0.0));
    let inner = bounds.inset(inset);
    let strips = [
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
    ];
    let mut corners = Vec::with_capacity(4);
    if inner.size.width > px(0.0) && inner.size.height > px(0.0) {
        let r = px(CANVAS_RADIUS)
            .min(inner.size.width * 0.5)
            .min(inner.size.height * 0.5);
        let c = r * (1.0 - 0.552_284_8);
        for (origin, x_sign, y_sign) in [
            (inner.origin, 1.0, 1.0),
            (inner.top_right(), -1.0, 1.0),
            (inner.bottom_right(), -1.0, -1.0),
            (inner.bottom_left(), 1.0, -1.0),
        ] {
            let at = |x: Pixels, y: Pixels| point(origin.x + x * x_sign, origin.y + y * y_sign);
            let mut path = PathBuilder::fill();
            path.move_to(origin);
            path.line_to(at(r, px(0.0)));
            path.cubic_bezier_to(at(px(0.0), r), at(c, px(0.0)), at(px(0.0), c));
            path.close();
            if let Ok(path) = path.build() {
                corners.push(path);
            }
        }
    }
    SurroundGeometry { strips, corners }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CaptionLayout {
    pub dock: f32,
    pub header: f32,
    pub hold_sidebar: bool,
}

/// One caption owner: dock its controls before reclaiming the global row;
/// restore that row before releasing the sidebar or moving controls out.
pub(super) struct CaptionDockMotion {
    dock: MotionValue,
    header: MotionValue,
    dock_target: f32,
    header_target: f32,
    layout: CaptionLayout,
}

impl CaptionDockMotion {
    pub fn new(docked: bool) -> Self {
        Self {
            dock: MotionValue::new(if docked { 1.0 } else { 0.0 }),
            header: MotionValue::new(if docked { 0.0 } else { 1.0 }),
            dock_target: if docked { 1.0 } else { 0.0 },
            header_target: if docked { 0.0 } else { 1.0 },
            layout: CaptionLayout {
                dock: if docked { 1.0 } else { 0.0 },
                header: if docked { 0.0 } else { 1.0 },
                hold_sidebar: docked,
            },
        }
    }

    pub fn current(&self) -> CaptionLayout {
        self.layout
    }

    pub fn value(
        &mut self,
        wants_dock: bool,
        ready: bool,
        window: &mut Window,
        cx: &App,
    ) -> CaptionLayout {
        let mut request = false;
        let layout = self.update_at(
            wants_dock,
            ready,
            Instant::now(),
            cx.reduce_motion(),
            || {
                request = true;
            },
        );
        if request {
            window.request_animation_frame();
        }
        layout
    }

    fn update_at(
        &mut self,
        wants_dock: bool,
        ready: bool,
        now: Instant,
        reduced: bool,
        mut request: impl FnMut(),
    ) -> CaptionLayout {
        if reduced {
            *self = Self::new(wants_dock && ready);
            return self.layout;
        }
        let can_reclaim = wants_dock && ready && self.layout.dock == 1.0;
        let header_target = if can_reclaim { 0.0 } else { 1.0 };
        if header_target != self.header_target {
            self.header_target = header_target;
            self.header
                .set_target_at(header_target, Duration::from_millis(140), now);
        }
        let header = self.header.update_at(now, reduced, &mut request);
        let may_move = header == 1.0;
        if may_move {
            let target = if wants_dock && ready { 1.0 } else { 0.0 };
            if target != self.dock_target {
                self.dock_target = target;
                self.dock
                    .set_target_at(target, Duration::from_millis(180), now);
            }
        }
        let dock = self.dock.update_at(now, reduced, &mut request);
        self.layout = CaptionLayout {
            dock,
            header,
            hold_sidebar: dock > 0.0 || header < 1.0,
        };
        // Reaching a phase endpoint still needs a frame to start its successor.
        if (wants_dock && ready && dock == 1.0 && header == 1.0)
            || (!wants_dock && dock > 0.0 && header == 1.0)
        {
            request();
        }
        self.layout
    }
}

pub(super) fn prepare_native_canvas(bounds: Bounds<Pixels>, window: &Window) {
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
            f64::from(CANVAS_RADIUS),
        )
    };
    if changed {
        window.request_native_surface_presentation_sync();
    }
}

unsafe extern "C" {
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
            let corners = HasWindowHandle::window_handle(window)
                .ok()
                .and_then(|handle| {
                    if let raw_window_handle::RawWindowHandle::AppKit(handle) = handle.as_raw() {
                        Some(unsafe {
                            con_terminal_canvas_header_corners(
                                handle.ns_view.as_ptr(),
                                f64::from(bounds.left()),
                                f64::from(bounds.top()),
                                f64::from(bounds.size.width),
                            )
                        })
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            let mut quad = gpui::fill(bounds, color);
            quad.corner_radii.top_left = px(if corners & 1 != 0 { CANVAS_RADIUS } else { 0.0 });
            quad.corner_radii.top_right = px(if corners & 2 != 0 { CANVAS_RADIUS } else { 0.0 });
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
    fn chrome_surround_never_paints_the_terminal_interior() {
        let geometry = surround_geometry(Bounds::new(
            point(px(10.0), px(20.0)),
            gpui::size(px(600.0), px(400.0)),
        ));
        assert_eq!(geometry.corners.len(), 4);
        for corner in &geometry.corners {
            assert!(corner.bounds.size.width <= px(CANVAS_RADIUS));
            assert!(corner.bounds.size.height <= px(CANVAS_RADIUS));
        }
        let contains = |x: f32, y: f32| {
            let p = point(px(x), px(y));
            geometry.strips.iter().any(|strip| strip.contains(&p))
                || geometry.corners.iter().any(|path| {
                    path.vertices.chunks_exact(3).any(|triangle| {
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
                        sides.iter().all(|side| *side >= 0.0)
                            || sides.iter().all(|side| *side <= 0.0)
                    })
                })
        };
        for (x, y) in [
            (12.0, 220.0),
            (608.0, 220.0),
            (310.0, 22.0),
            (310.0, 418.0),
            (16.5, 26.5),
        ] {
            assert!(contains(x, y), "missing chrome at {x},{y}");
        }
        for (x, y) in [(310.0, 220.0), (26.0, 28.0), (310.0, 28.0), (602.0, 220.0)] {
            assert!(!contains(x, y), "chrome painted terminal at {x},{y}");
        }
    }

    #[test]
    fn rapid_caption_reversals_keep_controls_out_of_the_terminal() {
        let start = Instant::now();
        let mut motion = CaptionDockMotion::new(true);
        for tick in 1..400 {
            let wants = tick % 17 < 8;
            let layout = motion.update_at(
                wants,
                true,
                start + Duration::from_millis(tick * 8),
                false,
                || {},
            );
            if layout.header < 1.0 {
                assert_eq!(layout.dock, 1.0);
                assert!(layout.hold_sidebar);
            }
        }
        for tick in 400..600 {
            motion.update_at(
                false,
                true,
                start + Duration::from_millis(tick * 8),
                false,
                || {},
            );
        }
        assert_eq!(motion.current().header, 1.0);
        assert_eq!(motion.current().dock, 0.0);
        assert!(!motion.current().hold_sidebar);
    }

    #[test]
    fn reduced_motion_resolves_without_intermediate_layout_or_frames() {
        let mut motion = CaptionDockMotion::new(false);
        for wants in [true, false, true] {
            let layout = motion.update_at(wants, true, Instant::now(), true, || {
                panic!("reduced motion scheduled a frame")
            });
            assert_eq!(layout.dock, if wants { 1.0 } else { 0.0 });
            assert_eq!(layout.header, if wants { 0.0 } else { 1.0 });
        }
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
                parent_ptr, 0.0, 0.0, 1.0, 1.0, 9.0
            ));
            con_terminal_canvas_apply_clip(host.cast());
            assert!(con_terminal_canvas_set_clip(
                parent_ptr, 0.0, 0.0, 0.0, 0.0, 9.0
            ));
            con_terminal_canvas_apply_clip(host.cast());
            let _: () = msg_send![host, release];
            let _: () = msg_send![parent, release];
        }
    }

    #[test]
    fn docking_restores_caption_before_releasing_sidebar_and_reverses_without_overlap() {
        let start = Instant::now();
        let mut motion = CaptionDockMotion::new(false);
        let mut tick = 0;
        for _ in 0..8 {
            for wants in [true, false, true, false] {
                for _ in 0..100 {
                    tick += 1;
                    let now = start + Duration::from_millis(tick * 8);
                    let layout = motion.update_at(wants, true, now, false, || {});
                    assert!((0.0..=1.0).contains(&layout.dock));
                    assert!((0.0..=1.0).contains(&layout.header));
                    if layout.header < 1.0 {
                        assert_eq!(layout.dock, 1.0, "controls crossed terminal content");
                        assert!(layout.hold_sidebar);
                    }
                }
            }
        }
    }

    #[test]
    fn caption_motion_is_idle_at_both_endpoints() {
        for docked in [false, true] {
            let mut motion = CaptionDockMotion::new(docked);
            let layout = motion.update_at(docked, true, Instant::now(), false, || {
                panic!("settled caption requested a frame")
            });
            assert_eq!(layout.dock, if docked { 1.0 } else { 0.0 });
        }
    }
}
