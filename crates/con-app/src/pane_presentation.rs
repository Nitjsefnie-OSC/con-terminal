//! Pane motion changes presentation rectangles, never a terminal's layout size.

use crate::motion::MotionValue;
use crate::terminal_pane::TerminalPane;
use gpui::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Part {
    Pane(usize),
    Divider(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Layout {
    Pane {
        id: usize,
        content: u64,
    },
    Split {
        id: usize,
        axis: Axis,
        ratio: f32,
        first: Box<Layout>,
        second: Box<Layout>,
    },
}

type Rects = HashMap<Part, Bounds<Pixels>>;

impl Layout {
    fn contains(&self, pane: usize) -> bool {
        match self {
            Self::Pane { id, .. } => *id == pane,
            Self::Split { first, second, .. } => first.contains(pane) || second.contains(pane),
        }
    }

    fn panes(&self, result: &mut HashMap<usize, u64>) {
        match self {
            Self::Pane { id, content } => {
                result.insert(*id, *content);
            }
            Self::Split { first, second, .. } => {
                first.panes(result);
                second.panes(result);
            }
        }
    }

    #[cfg(test)]
    fn geometry(&self, bounds: Bounds<Pixels>, zoom: Option<usize>, result: &mut Rects) {
        match self {
            Self::Pane { id, .. } => {
                result.insert(Part::Pane(*id), bounds);
            }
            Self::Split {
                id,
                axis,
                ratio,
                first,
                second,
            } => {
                let (ratio, gap) = if zoom.is_some_and(|pane| first.contains(pane)) {
                    (1.0, px(0.0))
                } else if zoom.is_some_and(|pane| second.contains(pane)) {
                    (0.0, px(0.0))
                } else {
                    (*ratio, extent(bounds, *axis).min(px(1.0)))
                };
                let first_extent = (extent(bounds, *axis) - gap).max(px(0.0)) * ratio;
                let mut a = bounds;
                let mut bar = bounds;
                let mut b = bounds;
                match axis {
                    Axis::Horizontal => {
                        a.size.width = first_extent;
                        bar.origin.x += first_extent;
                        bar.size.width = gap;
                        b.origin.x += first_extent + gap;
                        b.size.width = (bounds.size.width - first_extent - gap).max(px(0.0));
                    }
                    Axis::Vertical => {
                        a.size.height = first_extent;
                        bar.origin.y += first_extent;
                        bar.size.height = gap;
                        b.origin.y += first_extent + gap;
                        b.size.height = (bounds.size.height - first_extent - gap).max(px(0.0));
                    }
                }
                result.insert(Part::Divider(*id), bar);
                first.geometry(a, zoom, result);
                second.geometry(b, zoom, result);
            }
        }
    }

    fn parent(&self, pane: usize) -> Option<(usize, Axis, bool, &Layout)> {
        match self {
            Self::Pane { .. } => None,
            Self::Split {
                id,
                axis,
                first,
                second,
                ..
            } => {
                if matches!(first.as_ref(), Self::Pane { id, .. } if *id == pane) {
                    Some((*id, *axis, true, second))
                } else if matches!(second.as_ref(), Self::Pane { id, .. } if *id == pane) {
                    Some((*id, *axis, false, first))
                } else {
                    first.parent(pane).or_else(|| second.parent(pane))
                }
            }
        }
    }
}

fn extent(bounds: Bounds<Pixels>, axis: Axis) -> Pixels {
    match axis {
        Axis::Horizontal => bounds.size.width,
        Axis::Vertical => bounds.size.height,
    }
}

#[cfg(test)]
fn interpolate(a: Bounds<Pixels>, b: Bounds<Pixels>, progress: f32) -> Bounds<Pixels> {
    let x = a.left() + (b.left() - a.left()) * progress;
    let y = a.top() + (b.top() - a.top()) * progress;
    let right = a.right() + (b.right() - a.right()) * progress;
    let bottom = a.bottom() + (b.bottom() - a.bottom()) * progress;
    Bounds::new(
        point(x, y),
        size((right - x).max(px(0.0)), (bottom - y).max(px(0.0))),
    )
}

#[derive(Default)]
struct State {
    session: Option<u64>,
    layout: Option<Rc<Layout>>,
    cached_key: Option<(u64, u64)>,
    cached_layout: Option<Rc<Layout>>,
    zoom: Option<usize>,
    bounds: Option<Bounds<Pixels>>,
    scale: f32,
    tree: Option<AnimatedLayout>,
    presented: Rects,
    active: bool,
}

enum AnimatedLayout {
    Pane {
        id: usize,
    },
    Split {
        id: usize,
        axis: Axis,
        first: Box<Self>,
        second: Box<Self>,
        ratio: MotionValue,
        gap: MotionValue,
        ratio_target: f32,
        gap_target: f32,
        ratio_value: f32,
        gap_value: f32,
    },
}

impl AnimatedLayout {
    // Ignore departing leaves, but never animate an unrelated topology import
    // or reorder as if it were an ordinary split/close.
    fn matches_layout(&self, layout: &Layout) -> bool {
        match self {
            Self::Pane { id } => matches!(layout, Layout::Pane { id: other, .. } if id == other),
            Self::Split {
                id,
                axis,
                first,
                second,
                ..
            } => {
                if !first.alive(layout) {
                    return second.matches_layout(layout);
                }
                if !second.alive(layout) {
                    return first.matches_layout(layout);
                }
                match layout {
                    Layout::Split {
                        id: other,
                        axis: other_axis,
                        first: a,
                        second: b,
                        ..
                    } => {
                        id == other
                            && axis == other_axis
                            && first.matches_layout(a)
                            && second.matches_layout(b)
                    }
                    _ => false,
                }
            }
        }
    }

    fn new(layout: &Layout) -> Self {
        match layout {
            Layout::Pane { id, .. } => Self::Pane { id: *id },
            Layout::Split {
                id,
                axis,
                ratio,
                first,
                second,
            } => Self::Split {
                id: *id,
                axis: *axis,
                first: Box::new(Self::new(first)),
                second: Box::new(Self::new(second)),
                ratio: MotionValue::new(*ratio),
                gap: MotionValue::new(1.0),
                ratio_target: *ratio,
                gap_target: 1.0,
                ratio_value: *ratio,
                gap_value: 1.0,
            },
        }
    }

    fn contains(&self, pane: usize) -> bool {
        match self {
            Self::Pane { id } => *id == pane,
            Self::Split { first, second, .. } => first.contains(pane) || second.contains(pane),
        }
    }

    fn alive(&self, layout: &Layout) -> bool {
        match self {
            Self::Pane { id } => layout.contains(*id),
            Self::Split { first, second, .. } => first.alive(layout) || second.alive(layout),
        }
    }

    fn insert(
        &mut self,
        sibling: usize,
        added: usize,
        split_id: usize,
        axis: Axis,
        first_added: bool,
    ) -> bool {
        match self {
            Self::Pane { id } if *id == sibling => {
                let first = Self::Pane {
                    id: if first_added { added } else { sibling },
                };
                let second = Self::Pane {
                    id: if first_added { sibling } else { added },
                };
                let ratio_value = if first_added { 0.0 } else { 1.0 };
                *self = Self::Split {
                    id: split_id,
                    axis,
                    first: Box::new(first),
                    second: Box::new(second),
                    ratio: MotionValue::new(ratio_value),
                    gap: MotionValue::new(0.0),
                    ratio_target: ratio_value,
                    gap_target: 0.0,
                    ratio_value,
                    gap_value: 0.0,
                };
                true
            }
            Self::Split { first, second, .. } => {
                first.insert(sibling, added, split_id, axis, first_added)
                    || second.insert(sibling, added, split_id, axis, first_added)
            }
            _ => false,
        }
    }

    fn retarget(&mut self, layout: &Layout, zoom: Option<usize>, now: Instant, duration: Duration) {
        let Self::Split {
            id,
            axis: _,
            first,
            second,
            ratio,
            gap,
            ratio_target,
            gap_target,
            ..
        } = self
        else {
            return;
        };
        let next = if zoom.is_some_and(|id| first.contains(id)) {
            (1.0, 0.0)
        } else if zoom.is_some_and(|id| second.contains(id)) {
            (0.0, 0.0)
        } else if let Some(target) = find_split(layout, *id) {
            (target, 1.0)
        } else if first.alive(layout) {
            (1.0, 0.0)
        } else {
            (0.0, 0.0)
        };
        if *ratio_target != next.0 {
            ratio.set_target_at(next.0, duration, now);
            *ratio_target = next.0;
        }
        if *gap_target != next.1 {
            gap.set_target_at(next.1, duration, now);
            *gap_target = next.1;
        }
        first.retarget(layout, zoom, now, duration);
        second.retarget(layout, zoom, now, duration);
    }

    fn advance(&mut self, now: Instant, snap: bool) -> bool {
        match self {
            Self::Pane { .. } => false,
            Self::Split {
                first,
                second,
                ratio,
                gap,
                ratio_value,
                gap_value,
                ..
            } => {
                *ratio_value = ratio.update_at(now, snap, || {});
                *gap_value = gap.update_at(now, snap, || {});
                let first_active = first.advance(now, snap);
                let second_active = second.advance(now, snap);
                ratio.is_animating() || gap.is_animating() || first_active || second_active
            }
        }
    }

    fn prune(&mut self, layout: &Layout) {
        let Self::Split {
            first,
            second,
            ratio,
            gap,
            ratio_target,
            gap_target,
            ..
        } = self
        else {
            return;
        };
        first.prune(layout);
        second.prune(layout);
        if ratio.is_animating() || gap.is_animating() || *gap_target != 0.0 {
            return;
        }
        let take_first = *ratio_target == 1.0 && !second.alive(layout);
        let take_second = *ratio_target == 0.0 && !first.alive(layout);
        if take_first || take_second {
            let old = std::mem::replace(self, Self::Pane { id: usize::MAX });
            if let Self::Split { first, second, .. } = old {
                *self = if take_first { *first } else { *second };
            }
        }
    }

    fn geometry(&self, bounds: Bounds<Pixels>, result: &mut Rects) {
        match self {
            Self::Pane { id } => {
                result.insert(Part::Pane(*id), bounds);
            }
            Self::Split {
                id,
                axis,
                first,
                second,
                ratio_value,
                gap_value,
                ..
            } => {
                let gap = px(*gap_value).min(extent(bounds, *axis));
                let first_extent = (extent(bounds, *axis) - gap).max(px(0.0)) * *ratio_value;
                let mut a = bounds;
                let mut bar = bounds;
                let mut b = bounds;
                match axis {
                    Axis::Horizontal => {
                        a.size.width = first_extent;
                        bar.origin.x += first_extent;
                        bar.size.width = gap;
                        b.origin.x += first_extent + gap;
                        b.size.width = (bounds.size.width - first_extent - gap).max(px(0.0));
                    }
                    Axis::Vertical => {
                        a.size.height = first_extent;
                        bar.origin.y += first_extent;
                        bar.size.height = gap;
                        b.origin.y += first_extent + gap;
                        b.size.height = (bounds.size.height - first_extent - gap).max(px(0.0));
                    }
                }
                result.insert(Part::Divider(*id), bar);
                first.geometry(a, result);
                second.geometry(b, result);
            }
        }
    }
}

fn find_split(layout: &Layout, split: usize) -> Option<f32> {
    match layout {
        Layout::Pane { .. } => None,
        Layout::Split {
            id,
            ratio,
            first,
            second,
            ..
        } => {
            if *id == split {
                Some(*ratio)
            } else {
                find_split(first, split).or_else(|| find_split(second, split))
            }
        }
    }
}

impl State {
    #[allow(clippy::too_many_arguments)]
    fn begin_frame(
        &mut self,
        session: u64,
        layout: Rc<Layout>,
        zoom: Option<usize>,
        bounds: Bounds<Pixels>,
        scale: f32,
        animate: bool,
        now: Instant,
    ) -> bool {
        let same_layout = self.session == Some(session)
            && self
                .layout
                .as_ref()
                .is_some_and(|old| Rc::ptr_eq(old, &layout))
            && self.zoom == zoom;
        if same_layout && (self.bounds != Some(bounds) || self.scale != scale) {
            // Window/sidebar resizing is the hot path, not a topology change.
            // Reuse the tree and rectangle capacity; settle any pending motion.
            let tree = self
                .tree
                .as_mut()
                .expect("presentation tree is initialized");
            if self.active {
                tree.advance(now, true);
                tree.prune(&layout);
            }
            self.presented.clear();
            tree.geometry(bounds, &mut self.presented);
            self.bounds = Some(bounds);
            self.scale = scale;
            self.active = false;
            return false;
        }
        let changed = self.session != Some(session)
            || self
                .layout
                .as_ref()
                .is_none_or(|old| !Rc::ptr_eq(old, &layout))
            || self.zoom != zoom
            || self.bounds != Some(bounds)
            || self.scale != scale;
        if !changed && !self.active {
            return false;
        }
        if changed {
            let mut old_panes = HashMap::new();
            let mut panes = HashMap::new();
            if let Some(old) = &self.layout {
                old.panes(&mut old_panes);
            }
            layout.panes(&mut panes);
            let added: Vec<_> = panes
                .keys()
                .filter(|id| !old_panes.contains_key(id))
                .copied()
                .collect();
            let removed: Vec<_> = old_panes
                .keys()
                .filter(|id| !panes.contains_key(id))
                .copied()
                .collect();
            let same_content = panes
                .iter()
                .all(|(id, content)| old_panes.get(id).is_none_or(|old| old == content));
            let zoom_change = self.zoom != zoom && added.is_empty() && removed.is_empty();
            let split =
                added.len() == 1 && removed.is_empty() && self.zoom.is_none() && zoom.is_none();
            let close =
                removed.len() == 1 && added.is_empty() && self.zoom.is_none() && zoom.is_none();
            let mut can_animate = animate
                && same_content
                && self.session == Some(session)
                && self.bounds == Some(bounds)
                && self.scale == scale
                && (zoom_change || split || close);
            if can_animate && split {
                can_animate =
                    if let Some((split_id, axis, first_added, Layout::Pane { id: sibling, .. })) =
                        layout.parent(added[0])
                    {
                        self.tree.as_mut().is_some_and(|tree| {
                            tree.insert(*sibling, added[0], split_id, axis, first_added)
                        })
                    } else {
                        false
                    };
            }
            can_animate &= self
                .tree
                .as_ref()
                .is_some_and(|tree| tree.matches_layout(&layout));
            if !can_animate {
                self.tree = Some(AnimatedLayout::new(&layout));
            }
            let tree = self
                .tree
                .as_mut()
                .expect("presentation tree is initialized");
            tree.retarget(
                &layout,
                zoom,
                now,
                if can_animate {
                    Duration::from_millis(if close { 160 } else { 220 })
                } else {
                    Duration::ZERO
                },
            );
            tree.advance(now, !can_animate);
            self.layout = Some(layout.clone());
            self.session = Some(session);
            self.zoom = zoom;
            self.bounds = Some(bounds);
            self.scale = scale;
        }
        let tree = self
            .tree
            .as_mut()
            .expect("presentation tree is initialized");
        self.active = tree.advance(now, !animate);
        tree.prune(&layout);
        self.presented.clear();
        tree.geometry(bounds, &mut self.presented);
        self.active
    }
}

#[derive(Clone, Default)]
pub(crate) struct PaneMotion(Rc<RefCell<State>>);

impl PaneMotion {
    pub fn layout(&self, session: u64, key: u64, build: impl FnOnce() -> Layout) -> Rc<Layout> {
        let mut state = self.0.borrow_mut();
        if state.cached_key != Some((session, key)) {
            state.cached_layout = Some(Rc::new(build()));
            state.cached_key = Some((session, key));
        }
        state.cached_layout.as_ref().unwrap().clone()
    }

    fn viewport(&self, part: Part, fallback: Bounds<Pixels>, scale: f32) -> Bounds<Pixels> {
        let bounds = self
            .0
            .borrow()
            .presented
            .get(&part)
            .copied()
            .unwrap_or(fallback);
        crate::terminal_geometry::snap_terminal_bounds(bounds, scale)
    }
}

pub(crate) struct PaneScene {
    pub motion: PaneMotion,
    pub session: u64,
    pub layout: Rc<Layout>,
    pub zoom: Option<usize>,
    pub animate: bool,
    pub background: Hsla,
    pub child: AnyElement,
    pub frame_time: Instant,
}

impl IntoElement for PaneScene {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

fn fill_layout(window: &mut Window, cx: &mut App) -> LayoutId {
    let style = Style {
        size: size(relative(1.0).into(), relative(1.0).into()),
        ..Default::default()
    };
    window.request_layout(style, [], cx)
}

fn definite(size: Size<Pixels>) -> Size<AvailableSpace> {
    size.map(AvailableSpace::Definite)
}

impl Element for PaneScene {
    type RequestLayoutState = ();
    type PrepaintState = Vec<Bounds<Pixels>>;
    fn id(&self) -> Option<ElementId> {
        Some("pane-motion-scene".into())
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (fill_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Vec<Bounds<Pixels>> {
        let animate = self.animate && !cx.reduce_motion();
        let active = self.motion.0.borrow_mut().begin_frame(
            self.session,
            self.layout.clone(),
            self.zoom,
            bounds,
            window.scale_factor(),
            animate,
            self.frame_time,
        );
        if active {
            window.request_animation_frame();
        }
        let gaps = if active {
            self.motion
                .0
                .borrow()
                .presented
                .iter()
                .filter_map(|(&part, &rect)| {
                    let missing = match part {
                        Part::Pane(id) => {
                            !self.layout.contains(id) || self.zoom.is_some_and(|zoom| zoom != id)
                        }
                        Part::Divider(id) => {
                            self.zoom.is_some() || !contains_split(&self.layout, id)
                        }
                    };
                    (missing && rect.size.width > px(0.0) && rect.size.height > px(0.0))
                        .then_some(rect)
                })
                .collect()
        } else {
            Vec::new()
        };
        self.child
            .prepaint_as_root(bounds.origin, definite(bounds.size), window, cx);
        gaps
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        gaps: &mut Vec<Bounds<Pixels>>,
        window: &mut Window,
        cx: &mut App,
    ) {
        for &gap in gaps.iter() {
            window.paint_quad(fill(gap, self.background));
        }
        self.child.paint(window, cx);
    }
}

fn contains_split(node: &Layout, id: usize) -> bool {
    match node {
        Layout::Pane { .. } => false,
        Layout::Split {
            id: node_id,
            first,
            second,
            ..
        } => *node_id == id || contains_split(first, id) || contains_split(second, id),
    }
}

pub(crate) struct PanePresentation {
    pub motion: PaneMotion,
    pub pane: usize,
    pub header: AnyElement,
    pub body: AnyElement,
    pub terminal: Option<TerminalPane>,
    pub background: Hsla,
}

pub(crate) struct PanePaint {
    viewport: Bounds<Pixels>,
    header: Bounds<Pixels>,
    gaps: [Option<Bounds<Pixels>>; 4],
}

fn pane_body_geometry(
    bounds: Bounds<Pixels>,
    viewport: Bounds<Pixels>,
    header_height: Pixels,
    scale: f32,
) -> (Bounds<Pixels>, Bounds<Pixels>, Bounds<Pixels>) {
    let header = Bounds::new(
        viewport.origin,
        size(viewport.size.width, header_height.min(viewport.size.height)),
    );
    let body_viewport = crate::terminal_geometry::snap_terminal_bounds(
        Bounds::new(
            point(viewport.left(), header.bottom()),
            size(
                viewport.size.width,
                (viewport.size.height - header_height).max(px(0.0)),
            ),
        ),
        scale,
    );
    let final_body = Bounds::new(
        point(bounds.left(), bounds.top() + header_height),
        size(
            bounds.size.width,
            (bounds.size.height - header_height).max(px(0.0)),
        ),
    );
    let body_size = crate::terminal_geometry::snap_terminal_bounds(final_body, scale).size;
    let body_origin = point(
        body_viewport.left(),
        body_viewport.top() - (body_size.height - body_viewport.size.height).max(px(0.0)),
    );
    (header, body_viewport, Bounds::new(body_origin, body_size))
}

impl IntoElement for PanePresentation {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for PanePresentation {
    type RequestLayoutState = ();
    type PrepaintState = PanePaint;
    fn id(&self) -> Option<ElementId> {
        Some(("pane-presentation", self.pane).into())
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (fill_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> PanePaint {
        let viewport = self
            .motion
            .viewport(Part::Pane(self.pane), bounds, window.scale_factor());
        let header_height = self
            .header
            .layout_as_root(
                size(
                    AvailableSpace::Definite(bounds.size.width),
                    AvailableSpace::MinContent,
                ),
                window,
                cx,
            )
            .height
            .min(bounds.size.height);
        let (header, body_viewport, body_bounds) =
            pane_body_geometry(bounds, viewport, header_height, window.scale_factor());
        if let Some(terminal) = &self.terminal {
            terminal.set_presentation_viewport(Some(body_viewport), cx);
        }
        window.with_content_mask(Some(ContentMask { bounds: header }), |window| {
            self.header.prepaint_as_root(
                header.origin,
                size(
                    AvailableSpace::Definite(header.size.width),
                    AvailableSpace::MinContent,
                ),
                window,
                cx,
            );
        });
        window.with_content_mask(
            Some(ContentMask {
                bounds: body_viewport,
            }),
            |window| {
                self.body.prepaint_as_root(
                    body_bounds.origin,
                    definite(body_bounds.size),
                    window,
                    cx,
                );
            },
        );
        PanePaint {
            viewport: body_viewport,
            header,
            gaps: uncovered(body_viewport, body_bounds),
        }
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        state: &mut PanePaint,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(
            Some(ContentMask {
                bounds: state.header,
            }),
            |window| self.header.paint(window, cx),
        );
        window.with_content_mask(
            Some(ContentMask {
                bounds: state.viewport,
            }),
            |window| {
                for &gap in state.gaps.iter().flatten() {
                    window.paint_quad(fill(gap, self.background));
                }
                self.body.paint(window, cx);
            },
        );
    }
}

fn uncovered(viewport: Bounds<Pixels>, content: Bounds<Pixels>) -> [Option<Bounds<Pixels>>; 4] {
    let intersection = viewport.intersect(&content);
    if intersection.size.width <= px(0.0) || intersection.size.height <= px(0.0) {
        return [Some(viewport), None, None, None];
    }
    [
        (
            viewport.left(),
            viewport.top(),
            viewport.right(),
            intersection.top(),
        ),
        (
            viewport.left(),
            intersection.bottom(),
            viewport.right(),
            viewport.bottom(),
        ),
        (
            viewport.left(),
            intersection.top(),
            intersection.left(),
            intersection.bottom(),
        ),
        (
            intersection.right(),
            intersection.top(),
            viewport.right(),
            intersection.bottom(),
        ),
    ]
    .map(|(left, top, right, bottom)| {
        (right > left && bottom > top)
            .then(|| Bounds::new(point(left, top), size(right - left, bottom - top)))
    })
}

pub(crate) struct DividerPresentation {
    pub motion: PaneMotion,
    pub split: usize,
    pub axis: Axis,
    pub child: AnyElement,
}
impl IntoElement for DividerPresentation {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for DividerPresentation {
    type RequestLayoutState = ();
    type PrepaintState = Bounds<Pixels>;
    fn id(&self) -> Option<ElementId> {
        Some(("divider-presentation", self.split).into())
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        // The child must start as an independent root. Re-rooting a layout ID
        // already measured under a parent can reuse GPUI's cached absolute
        // origin and offset the resize hitbox a second time.
        let style = Style {
            size: match self.axis {
                Axis::Horizontal => size(px(1.0).into(), relative(1.0).into()),
                Axis::Vertical => size(relative(1.0).into(), px(1.0).into()),
            },
            flex_shrink: 0.0,
            ..Default::default()
        };
        (window.request_layout(style, [], cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Bounds<Pixels> {
        let viewport =
            self.motion
                .viewport(Part::Divider(self.split), bounds, window.scale_factor());
        if viewport.size.width > px(0.0) && viewport.size.height > px(0.0) {
            // Keep the existing five-point resize hit target outside the
            // one-point painted divider. Only a collapsed divider has no hitbox.
            self.child
                .prepaint_as_root(viewport.origin, definite(viewport.size), window, cx);
        } else {
            window.with_content_mask(Some(ContentMask { bounds: viewport }), |window| {
                self.child
                    .prepaint_as_root(viewport.origin, definite(viewport.size), window, cx);
            });
        }
        viewport
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        viewport: &mut Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(Some(ContentMask { bounds: *viewport }), |window| {
            self.child.paint(window, cx)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::prelude::v1::test;

    fn pane(id: usize) -> Layout {
        Layout::Pane {
            id,
            content: id as u64,
        }
    }
    fn split(id: usize, axis: Axis, first: Layout, second: Layout) -> Layout {
        Layout::Split {
            id,
            axis,
            ratio: 0.5,
            first: Box::new(first),
            second: Box::new(second),
        }
    }
    fn bounds() -> Bounds<Pixels> {
        Bounds::new(point(px(12.0), px(34.0)), size(px(1000.0), px(700.0)))
    }

    fn assert_tiled(state: &State) {
        let root = state.bounds.unwrap();
        let tiles: Vec<_> = state
            .presented
            .iter()
            .filter_map(|(_, bounds)| {
                (bounds.size.width > px(0.0) && bounds.size.height > px(0.0)).then_some(*bounds)
            })
            .collect();
        let area: f32 = tiles
            .iter()
            .map(|r| r.size.width.as_f32() * r.size.height.as_f32())
            .sum();
        assert!(
            (area - root.size.width.as_f32() * root.size.height.as_f32()).abs() < 1.0,
            "pane scene has gaps or duplicated coverage: {tiles:?}"
        );
        for (index, a) in tiles.iter().enumerate() {
            for b in &tiles[index + 1..] {
                let overlap = a.intersect(b);
                assert!(
                    overlap.size.width <= px(0.002) || overlap.size.height <= px(0.002),
                    "pane scene overlaps: {a:?}, {b:?}"
                );
            }
        }
    }

    #[test]
    fn split_and_close_keep_one_tiled_scene_in_both_directions() {
        for axis in [Axis::Horizontal, Axis::Vertical] {
            for before in [false, true] {
                let t = Instant::now();
                let mut state = State::default();
                let one = Rc::new(pane(0));
                let two = Rc::new(if before {
                    split(0, axis, pane(1), pane(0))
                } else {
                    split(0, axis, pane(0), pane(1))
                });
                assert!(!state.begin_frame(1, one.clone(), None, bounds(), 2.0, true, t));
                assert!(state.begin_frame(1, two.clone(), None, bounds(), 2.0, true, t));
                for millis in [0, 16, 50, 120, 220] {
                    state.begin_frame(
                        1,
                        two.clone(),
                        None,
                        bounds(),
                        2.0,
                        true,
                        t + Duration::from_millis(millis),
                    );
                    assert_tiled(&state);
                }
                let closing = t + Duration::from_millis(300);
                assert!(state.begin_frame(1, one.clone(), None, bounds(), 2.0, true, closing));
                for millis in [0, 16, 50, 100, 160] {
                    state.begin_frame(
                        1,
                        one.clone(),
                        None,
                        bounds(),
                        2.0,
                        true,
                        closing + Duration::from_millis(millis),
                    );
                    assert_tiled(&state);
                }
                assert_eq!(state.presented.len(), 1);
                let mut expected = Rects::new();
                one.geometry(bounds(), None, &mut expected);
                assert_eq!(state.presented, expected);
            }
        }
    }

    #[test]
    fn nested_close_and_zoom_reversal_preserve_coverage() {
        let t = Instant::now();
        let mut state = State::default();
        let three = Rc::new(split(
            0,
            Axis::Horizontal,
            pane(0),
            split(1, Axis::Vertical, pane(1), pane(2)),
        ));
        state.begin_frame(1, three.clone(), None, bounds(), 2.0, true, t);
        state.begin_frame(1, three.clone(), Some(2), bounds(), 2.0, true, t);
        for millis in [16, 100] {
            state.begin_frame(
                1,
                three.clone(),
                Some(2),
                bounds(),
                2.0,
                true,
                t + Duration::from_millis(millis),
            );
            assert_tiled(&state);
        }
        let displayed = state.presented.clone();
        state.begin_frame(
            1,
            three.clone(),
            None,
            bounds(),
            2.0,
            true,
            t + Duration::from_millis(100),
        );
        assert_eq!(state.presented, displayed);
        for millis in [116, 180, 320] {
            state.begin_frame(
                1,
                three.clone(),
                None,
                bounds(),
                2.0,
                true,
                t + Duration::from_millis(millis),
            );
            assert_tiled(&state);
        }
        let two = Rc::new(split(0, Axis::Horizontal, pane(0), pane(2)));
        let closing = t + Duration::from_millis(400);
        state.begin_frame(1, two.clone(), None, bounds(), 2.0, true, closing);
        for millis in [0, 16, 100, 160] {
            state.begin_frame(
                1,
                two.clone(),
                None,
                bounds(),
                2.0,
                true,
                closing + Duration::from_millis(millis),
            );
            assert_tiled(&state);
        }
        assert!(!state.presented.contains_key(&Part::Pane(1)));
    }

    #[test]
    fn interruption_does_not_accumulate_opaque_or_overlapping_departures() {
        let t = Instant::now();
        let mut state = State::default();
        let three = Rc::new(split(
            0,
            Axis::Horizontal,
            pane(0),
            split(1, Axis::Vertical, pane(1), pane(2)),
        ));
        let two = Rc::new(split(0, Axis::Horizontal, pane(0), pane(2)));
        let one = Rc::new(pane(2));
        state.begin_frame(1, three, None, bounds(), 2.0, true, t);
        state.begin_frame(1, two.clone(), None, bounds(), 2.0, true, t);
        state.begin_frame(
            1,
            two,
            None,
            bounds(),
            2.0,
            true,
            t + Duration::from_millis(30),
        );
        state.begin_frame(
            1,
            one.clone(),
            None,
            bounds(),
            2.0,
            true,
            t + Duration::from_millis(30),
        );
        for millis in [30, 46, 100, 190] {
            state.begin_frame(
                1,
                one.clone(),
                None,
                bounds(),
                2.0,
                true,
                t + Duration::from_millis(millis),
            );
            assert_tiled(&state);
        }
    }

    #[test]
    fn resizing_switching_sessions_content_and_reduce_motion_snap() {
        let t = Instant::now();
        let mut state = State::default();
        let one = Rc::new(pane(0));
        let two = Rc::new(split(0, Axis::Horizontal, pane(0), pane(1)));
        state.begin_frame(1, one, None, bounds(), 2.0, true, t);
        state.begin_frame(1, two.clone(), None, bounds(), 2.0, true, t);
        let mut resized = bounds();
        resized.size.width = px(1200.0);
        assert!(!state.begin_frame(1, two.clone(), None, resized, 2.0, true, t));
        assert_tiled(&state);
        let mut expected = Rects::new();
        two.geometry(resized, None, &mut expected);
        assert_eq!(state.presented, expected);
        assert!(!state.begin_frame(2, two.clone(), Some(1), resized, 2.0, true, t));
        assert!(state.begin_frame(2, two.clone(), None, resized, 2.0, true, t));
        assert!(!state.begin_frame(2, two.clone(), None, resized, 2.0, false, t));
        assert_tiled(&state);
        let changed = Rc::new(split(
            0,
            Axis::Horizontal,
            pane(0),
            Layout::Pane {
                id: 1,
                content: 999,
            },
        ));
        assert!(!state.begin_frame(2, changed, Some(1), resized, 2.0, true, t));
    }

    #[test]
    fn body_grid_size_is_fixed_across_every_animation_origin_and_scale() {
        let target = Bounds::new(point(px(73.25), px(81.75)), size(px(479.25), px(311.25)));
        let from = Bounds::new(point(px(12.0), px(34.0)), size(px(1000.0), px(700.0)));
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let mut previous_size = None;
            for step in 0..=120 {
                let viewport = crate::terminal_geometry::snap_terminal_bounds(
                    interpolate(from, target, step as f32 / 120.0),
                    scale,
                );
                let (header, body_viewport, body) =
                    pane_body_geometry(target, viewport, px(24.0), scale);
                assert_eq!(*previous_size.get_or_insert(body.size), body.size);
                assert_eq!(header.left(), viewport.left());
                assert_eq!(body.left(), body_viewport.left());
                assert!(body.top() <= body_viewport.top());
                for gap in uncovered(body_viewport, body).into_iter().flatten() {
                    let overlap = gap.intersect(&body);
                    assert!(
                        overlap.size.width <= px(0.0) || overlap.size.height <= px(0.0),
                        "glass background painted twice"
                    );
                }
            }
        }
    }

    #[test]
    fn idle_layout_reuses_descriptor_and_requests_no_animation_frames() {
        let motion = PaneMotion::default();
        let builds = std::cell::Cell::new(0);
        let t = Instant::now();
        for frame in 0..100 {
            let layout = motion.layout(1, 10, || {
                builds.set(builds.get() + 1);
                pane(0)
            });
            assert!(!motion.0.borrow_mut().begin_frame(
                1,
                layout,
                None,
                bounds(),
                2.0,
                true,
                t + Duration::from_millis(frame)
            ));
        }
        assert_eq!(builds.get(), 1);
    }

    struct SceneFixture {
        motion: PaneMotion,
        layout: Rc<Layout>,
        time: Instant,
        samples: Rc<RefCell<Vec<(usize, Bounds<Pixels>)>>>,
        divider_downs: Rc<std::cell::Cell<usize>>,
    }

    impl Render for SceneFixture {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            fn node(
                layout: &Layout,
                motion: &PaneMotion,
                samples: &Rc<RefCell<Vec<(usize, Bounds<Pixels>)>>>,
                divider_downs: &Rc<std::cell::Cell<usize>>,
            ) -> AnyElement {
                match layout {
                    Layout::Pane { id, .. } => {
                        let id = *id;
                        let samples = samples.clone();
                        PanePresentation {
                            motion: motion.clone(),
                            pane: id,
                            terminal: None,
                            background: Hsla::default(),
                            header: div().h(px(24.0)).w_full().into_any_element(),
                            body: canvas(
                                move |bounds, _, _| {
                                    samples.borrow_mut().push((id, bounds));
                                },
                                |_, _, _, _| {},
                            )
                            .size_full()
                            .into_any_element(),
                        }
                        .into_any_element()
                    }
                    Layout::Split {
                        id,
                        axis,
                        ratio,
                        first,
                        second,
                    } => {
                        let leaf = |layout: &Layout, ratio: f32| {
                            div()
                                .flex_grow_1()
                                .flex_shrink_1()
                                .flex_basis(relative(ratio))
                                .min_w_0()
                                .min_h_0()
                                .child(node(layout, motion, samples, divider_downs))
                        };
                        let divider = if *axis == Axis::Horizontal {
                            div().w(px(1.0)).h_full()
                        } else {
                            div().h(px(1.0)).w_full()
                        };
                        let hits = divider_downs.clone();
                        let handle = if *axis == Axis::Horizontal {
                            div()
                                .absolute()
                                .left(px(-2.0))
                                .top_0()
                                .bottom_0()
                                .w(px(5.0))
                        } else {
                            div().absolute().top(px(-2.0)).left_0().right_0().h(px(5.0))
                        }
                        .debug_selector(|| "motion-divider-handle".into())
                        .on_mouse_down(
                            MouseButton::Left,
                            move |_, _, cx| {
                                hits.set(hits.get() + 1);
                                cx.stop_propagation();
                            },
                        );
                        let divider = divider.relative().child(handle);
                        let root = if *axis == Axis::Horizontal {
                            div().flex().flex_row()
                        } else {
                            div().flex().flex_col()
                        };
                        root.size_full()
                            .child(leaf(first, *ratio))
                            .child(DividerPresentation {
                                motion: motion.clone(),
                                split: *id,
                                axis: *axis,
                                child: divider.flex_shrink_0().into_any_element(),
                            })
                            .child(leaf(second, 1.0 - ratio))
                            .into_any_element()
                    }
                }
            }
            div().w(px(1000.0)).h(px(700.0)).child(PaneScene {
                motion: self.motion.clone(),
                session: 1,
                layout: self.layout.clone(),
                zoom: None,
                animate: true,
                background: Hsla::default(),
                frame_time: self.time,
                child: node(
                    &self.layout,
                    &self.motion,
                    &self.samples,
                    &self.divider_downs,
                ),
            })
        }
    }

    #[gpui::test]
    fn real_layout_keeps_live_body_size_fixed_while_viewport_moves(cx: &mut TestAppContext) {
        let t = Instant::now();
        let samples = Rc::new(RefCell::new(Vec::new()));
        let (root, cx) = cx.add_window_view(|_, _| SceneFixture {
            motion: PaneMotion::default(),
            layout: Rc::new(pane(0)),
            time: t,
            samples: samples.clone(),
            divider_downs: Rc::default(),
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        root.update(cx, |root, cx| {
            root.layout = Rc::new(split(0, Axis::Horizontal, pane(0), pane(1)));
            cx.notify();
        });
        let mut body_sizes = HashMap::new();
        let mut poses = Vec::new();
        for millis in [0, 16, 50, 100, 220] {
            samples.borrow_mut().clear();
            root.update(cx, |root, cx| {
                root.time = t + Duration::from_millis(millis);
                cx.notify();
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            for (id, bounds) in samples.borrow().iter() {
                assert_eq!(*body_sizes.entry(*id).or_insert(bounds.size), bounds.size);
                assert!(bounds.size.width > px(490.0));
                assert_eq!(bounds.size.height, px(676.0));
            }
            root.read_with(cx, |root, _| {
                let state = root.motion.0.borrow();
                assert_tiled(&state);
                poses.push(state.presented[&Part::Pane(1)]);
            });
        }
        assert!(poses[0].size.width < poses[2].size.width);
        assert!(poses[2].size.width < poses[4].size.width);
        assert_eq!(body_sizes.len(), 2);
        let handle = cx.debug_bounds("motion-divider-handle").unwrap();
        root.read_with(cx, |root, _| {
            let state = root.motion.0.borrow();
            let divider = state.presented[&Part::Divider(0)];
            assert_eq!(handle.center().x, divider.center().x);
            assert_eq!(handle.size.height, divider.size.height);
        });
        cx.simulate_mouse_down(handle.center(), MouseButton::Left, Modifiers::default());
        root.read_with(cx, |root, _| assert_eq!(root.divider_downs.get(), 1));
    }
}
