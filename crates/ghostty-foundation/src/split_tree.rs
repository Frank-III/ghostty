//! Immutable split tree for terminal panes, ported from `src/datastruct/split_tree.zig`.
//!
//! Diagram formatting and GTK GObject glue are omitted.

use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeHandle(u16);

impl NodeHandle {
    pub const ROOT: Self = Self(0);

    pub fn idx(self) -> usize {
        self.0 as usize
    }

    fn offset(self, amount: usize) -> Self {
        Self((self.0 as usize + amount) as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitLayout {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitNode {
    pub layout: SplitLayout,
    pub ratio: f32,
    pub left: NodeHandle,
    pub right: NodeHandle,
}

#[derive(Debug)]
pub enum Node<V> {
    Leaf(Rc<V>),
    Split(SplitNode),
}

impl<V> Clone for Node<V> {
    fn clone(&self) -> Self {
        match self {
            Self::Leaf(view) => Self::Leaf(Rc::clone(view)),
            Self::Split(split) => Self::Split(*split),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SplitTree<V> {
    nodes: Vec<Node<V>>,
    zoomed: Option<NodeHandle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialSlot {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl SpatialSlot {
    pub fn max_x(self) -> f32 {
        self.x + self.width
    }

    pub fn max_y(self) -> f32 {
        self.y + self.height
    }
}

/// Normalized 1×1 spatial layout aligned with tree node indices.
#[derive(Debug, Clone)]
pub struct Spatial {
    pub slots: Vec<SpatialSlot>,
}

impl Spatial {
    pub const EMPTY: Self = Self { slots: Vec::new() };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Goto {
    Previous,
    Next,
    PreviousWrapped,
    NextWrapped,
    Spatial(SpatialDirection),
}

#[derive(Debug, Clone)]
pub struct ViewEntry<'a, V> {
    pub handle: NodeHandle,
    pub view: &'a V,
}

pub struct SplitTreeIterator<'a, V> {
    index: usize,
    nodes: &'a [Node<V>],
}

impl<'a, V> Iterator for SplitTreeIterator<'a, V> {
    type Item = ViewEntry<'a, V>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.nodes.len() {
            let handle = NodeHandle(self.index as u16);
            self.index += 1;
            if let Node::Leaf(view) = &self.nodes[handle.idx()] {
                return Some(ViewEntry {
                    handle,
                    view: view.as_ref(),
                });
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Dimensions {
    width: u16,
    height: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Backtrack {
    Deadend,
    Backtrack,
    Result(NodeHandle),
}

impl<V> SplitTree<V> {
    pub fn new(view: Rc<V>) -> Self {
        Self {
            nodes: vec![Node::Leaf(view)],
            zoomed: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            zoomed: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn is_split(&self) -> bool {
        !self.is_empty() && matches!(self.nodes[0], Node::Split(_))
    }

    pub fn nodes(&self) -> &[Node<V>] {
        &self.nodes
    }

    pub fn zoomed(&self) -> Option<NodeHandle> {
        self.zoomed
    }

    pub fn zoom(&mut self, handle: Option<NodeHandle>) {
        if let Some(h) = handle {
            debug_assert!(h.idx() < self.nodes.len());
        }
        self.zoomed = handle;
    }

    pub fn iterator(&self) -> SplitTreeIterator<'_, V> {
        SplitTreeIterator {
            index: 0,
            nodes: &self.nodes,
        }
    }

    pub fn clone_tree(&self) -> Self {
        if self.is_empty() {
            return Self::empty();
        }
        Self {
            nodes: self.nodes.clone(),
            zoomed: self.zoomed,
        }
    }

    pub fn deepest(&self, side: Side, from: NodeHandle) -> NodeHandle {
        let mut current = from;
        loop {
            match &self.nodes[current.idx()] {
                Node::Leaf(_) => return current,
                Node::Split(split) => {
                    current = match side {
                        Side::Left => split.left,
                        Side::Right => split.right,
                    };
                }
            }
        }
    }

    pub fn previous(&self, from: NodeHandle) -> Option<NodeHandle> {
        match self.previous_backtrack(from, NodeHandle::ROOT) {
            Backtrack::Result(v) => Some(v),
            Backtrack::Backtrack | Backtrack::Deadend => None,
        }
    }

    pub fn next(&self, from: NodeHandle) -> Option<NodeHandle> {
        match self.next_backtrack(from, NodeHandle::ROOT) {
            Backtrack::Result(v) => Some(v),
            Backtrack::Backtrack | Backtrack::Deadend => None,
        }
    }

    pub fn previous_wrapped(&self, from: NodeHandle) -> NodeHandle {
        self.previous(from)
            .unwrap_or_else(|| self.deepest(Side::Right, NodeHandle::ROOT))
    }

    pub fn next_wrapped(&self, from: NodeHandle) -> NodeHandle {
        self.next(from)
            .unwrap_or_else(|| self.deepest(Side::Left, NodeHandle::ROOT))
    }

    pub fn goto(&self, from: NodeHandle, to: Goto) -> Option<NodeHandle> {
        match to {
            Goto::Previous => self.previous(from),
            Goto::Next => self.next(from),
            Goto::PreviousWrapped => Some(self.previous_wrapped(from)),
            Goto::NextWrapped => Some(self.next_wrapped(from)),
            Goto::Spatial(direction) => self.nearest_wrapped(from, direction),
        }
    }

    pub fn spatial(&self) -> Spatial {
        if self.nodes.is_empty() {
            return Spatial::EMPTY;
        }

        let dim = self.dimensions(NodeHandle::ROOT);
        let dim_w = dim.width as f32;
        let dim_h = dim.height as f32;

        let mut slots = vec![
            SpatialSlot {
                x: 0.0,
                y: 0.0,
                width: dim_w,
                height: dim_h,
            };
            self.nodes.len()
        ];
        self.fill_spatial_slots(&mut slots, NodeHandle::ROOT);

        for slot in &mut slots {
            slot.x /= dim_w;
            slot.y /= dim_h;
            slot.width /= dim_w;
            slot.height /= dim_h;
        }

        Spatial { slots }
    }

    pub fn equalize(&self) -> Self {
        if self.is_empty() {
            return Self::empty();
        }

        let mut nodes = self.nodes.clone();
        for node in &mut nodes {
            if let Node::Split(split) = node {
                let weight_left = self.weight(split.left, split.layout, 0);
                let weight_right = self.weight(split.right, split.layout, 0);
                debug_assert!(weight_left > 0 && weight_right > 0);
                let total = (weight_left + weight_right) as f32;
                split.ratio = weight_left as f32 / total;
            }
        }

        Self {
            nodes,
            zoomed: self.zoomed,
        }
    }

    pub fn resize(&self, from: NodeHandle, layout: SplitLayout, ratio: f32) -> Self {
        debug_assert!((-1.0..=1.0).contains(&ratio));
        debug_assert!(ratio.is_finite());

        if self.is_empty() {
            return Self::empty();
        }

        let mut result = self.clone_tree();

        let parent_handle = match self.find_parent_split(layout, from, NodeHandle::ROOT) {
            Backtrack::Result(v) => v,
            Backtrack::Backtrack | Backtrack::Deadend => return result,
        };

        let sp = result.spatial();
        let scale = match layout {
            SplitLayout::Horizontal => sp.slots[parent_handle.idx()].width / sp.slots[0].width,
            SplitLayout::Vertical => sp.slots[parent_handle.idx()].height / sp.slots[0].height,
        };

        if scale == 0.0 {
            return result;
        }

        if let Node::Split(split) = &mut result.nodes[parent_handle.idx()] {
            let new_ratio = (split.ratio + ratio / scale).clamp(0.0, 1.0);
            split.ratio = new_ratio;
        }

        result
    }

    pub fn resize_in_place(&mut self, at: NodeHandle, ratio: f32) {
        if let Node::Split(split) = &mut self.nodes[at.idx()] {
            split.ratio = ratio;
        } else {
            debug_assert!(false, "resize_in_place requires a split node");
        }
    }

    pub fn split(
        &self,
        at: NodeHandle,
        direction: SplitDirection,
        ratio: f32,
        insert: &Self,
    ) -> Self {
        assert!(!self.is_empty());
        assert!(!insert.is_empty());

        let mut nodes = Vec::with_capacity(self.nodes.len() + insert.nodes.len() + 1);
        nodes.extend(self.nodes.iter().cloned());

        let insert_offset = self.nodes.len();
        for node in &insert.nodes {
            nodes.push(match node {
                Node::Leaf(v) => Node::Leaf(Rc::clone(v)),
                Node::Split(s) => Node::Split(SplitNode {
                    layout: s.layout,
                    ratio: s.ratio,
                    left: s.left.offset(insert_offset),
                    right: s.right.offset(insert_offset),
                }),
            });
        }

        let (layout, left_first) = match direction {
            SplitDirection::Left => (SplitLayout::Horizontal, true),
            SplitDirection::Right => (SplitLayout::Horizontal, false),
            SplitDirection::Up => (SplitLayout::Vertical, true),
            SplitDirection::Down => (SplitLayout::Vertical, false),
        };

        let moved = nodes[at.idx()].clone();
        let new_leaf_idx = nodes.len();
        nodes.push(moved);
        nodes[at.idx()] = Node::Split(SplitNode {
            layout,
            ratio,
            left: NodeHandle(if left_first {
                insert_offset as u16
            } else {
                new_leaf_idx as u16
            }),
            right: NodeHandle(if left_first {
                new_leaf_idx as u16
            } else {
                insert_offset as u16
            }),
        });

        Self {
            nodes,
            zoomed: None,
        }
    }

    pub fn remove(&self, at: NodeHandle) -> Self {
        assert!(at.idx() < self.nodes.len());
        if at == NodeHandle::ROOT {
            return Self::empty();
        }

        let count = self.count_after_removal(NodeHandle::ROOT, at, 0);
        let mut slots: Vec<Option<Node<V>>> = (0..count).map(|_| None).collect();
        let mut zoomed = None;
        let written = self.remove_node(&mut slots, &mut zoomed, 0, NodeHandle::ROOT, at);
        debug_assert!(written == count);
        let nodes = slots
            .into_iter()
            .map(|node| node.expect("remove_node slot"))
            .collect();
        Self { nodes, zoomed }
    }

    fn previous_backtrack(&self, from: NodeHandle, current: NodeHandle) -> Backtrack {
        if from == current {
            return Backtrack::Backtrack;
        }

        match &self.nodes[current.idx()] {
            Node::Leaf(_) => Backtrack::Deadend,
            Node::Split(split) => match self.previous_backtrack(from, split.left) {
                Backtrack::Result(v) => Backtrack::Result(v),
                Backtrack::Backtrack => Backtrack::Backtrack,
                Backtrack::Deadend => match self.previous_backtrack(from, split.right) {
                    Backtrack::Result(v) => Backtrack::Result(v),
                    Backtrack::Deadend => Backtrack::Deadend,
                    Backtrack::Backtrack => {
                        Backtrack::Result(self.deepest(Side::Right, split.left))
                    }
                },
            },
        }
    }

    fn dimensions(&self, current: NodeHandle) -> Dimensions {
        match &self.nodes[current.idx()] {
            Node::Leaf(_) => Dimensions {
                width: 1,
                height: 1,
            },
            Node::Split(split) => {
                let left = self.dimensions(split.left);
                let right = self.dimensions(split.right);
                match split.layout {
                    SplitLayout::Horizontal => Dimensions {
                        width: left.width + right.width,
                        height: left.height.max(right.height),
                    },
                    SplitLayout::Vertical => Dimensions {
                        width: left.width.max(right.width),
                        height: left.height + right.height,
                    },
                }
            }
        }
    }

    fn fill_spatial_slots(&self, slots: &mut [SpatialSlot], current: NodeHandle) {
        let current_idx = current.idx();
        debug_assert!(slots[current_idx].width >= 0.0 && slots[current_idx].height >= 0.0);

        match &self.nodes[current_idx] {
            Node::Leaf(_) => {}
            Node::Split(split) => {
                let parent = slots[current_idx];
                match split.layout {
                    SplitLayout::Horizontal => {
                        slots[split.left.idx()] = SpatialSlot {
                            x: parent.x,
                            y: parent.y,
                            width: parent.width * split.ratio,
                            height: parent.height,
                        };
                        slots[split.right.idx()] = SpatialSlot {
                            x: parent.x + parent.width * split.ratio,
                            y: parent.y,
                            width: parent.width * (1.0 - split.ratio),
                            height: parent.height,
                        };
                    }
                    SplitLayout::Vertical => {
                        slots[split.left.idx()] = SpatialSlot {
                            x: parent.x,
                            y: parent.y,
                            width: parent.width,
                            height: parent.height * split.ratio,
                        };
                        slots[split.right.idx()] = SpatialSlot {
                            x: parent.x,
                            y: parent.y + parent.height * split.ratio,
                            width: parent.width,
                            height: parent.height * (1.0 - split.ratio),
                        };
                    }
                }
                self.fill_spatial_slots(slots, split.left);
                self.fill_spatial_slots(slots, split.right);
            }
        }
    }

    fn nearest(
        &self,
        sp: &Spatial,
        from: NodeHandle,
        direction: SpatialDirection,
        target: SpatialSlot,
    ) -> Option<NodeHandle> {
        let mut result: Option<(NodeHandle, f32)> = None;

        for (handle, slot) in sp.slots.iter().enumerate() {
            if handle == from.idx() {
                continue;
            }

            if !matches!(self.nodes[handle], Node::Leaf(_)) {
                continue;
            }

            let in_direction = match direction {
                SpatialDirection::Left => slot.max_x() <= target.x,
                SpatialDirection::Right => slot.x >= target.max_x(),
                SpatialDirection::Up => slot.max_y() <= target.y,
                SpatialDirection::Down => slot.y >= target.max_y(),
            };
            if !in_direction {
                continue;
            }

            let dx = slot.x - target.x;
            let dy = slot.y - target.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if let Some((_, best)) = result {
                if distance >= best {
                    continue;
                }
            }
            result = Some((NodeHandle(handle as u16), distance));
        }

        result.map(|(handle, _)| handle)
    }

    fn nearest_wrapped(&self, from: NodeHandle, direction: SpatialDirection) -> Option<NodeHandle> {
        let sp = self.spatial();
        let mut target = sp.slots[from.idx()];
        if let Some(handle) = self.nearest(&sp, from, direction, target) {
            return Some(handle);
        }

        debug_assert!(target.x >= 0.0 && target.y >= 0.0);
        debug_assert!(target.max_x() <= 1.0 && target.max_y() <= 1.0);

        match direction {
            SpatialDirection::Left => target.x += 1.0,
            SpatialDirection::Right => target.x -= 1.0,
            SpatialDirection::Up => target.y += 1.0,
            SpatialDirection::Down => target.y -= 1.0,
        }

        self.nearest(&sp, from, direction, target)
    }

    fn weight(&self, from: NodeHandle, layout: SplitLayout, acc: usize) -> usize {
        match &self.nodes[from.idx()] {
            Node::Leaf(_) => acc + 1,
            Node::Split(split) => {
                if split.layout == layout {
                    self.weight(split.left, layout, acc) + self.weight(split.right, layout, acc)
                } else {
                    1
                }
            }
        }
    }

    fn find_parent_split(
        &self,
        layout: SplitLayout,
        from: NodeHandle,
        current: NodeHandle,
    ) -> Backtrack {
        if from == current {
            return Backtrack::Backtrack;
        }

        match &self.nodes[current.idx()] {
            Node::Leaf(_) => Backtrack::Deadend,
            Node::Split(split) => match self.find_parent_split(layout, from, split.left) {
                Backtrack::Result(v) => Backtrack::Result(v),
                Backtrack::Backtrack => {
                    if split.layout == layout {
                        Backtrack::Result(current)
                    } else {
                        Backtrack::Backtrack
                    }
                }
                Backtrack::Deadend => match self.find_parent_split(layout, from, split.right) {
                    Backtrack::Deadend => Backtrack::Deadend,
                    Backtrack::Result(v) => Backtrack::Result(v),
                    Backtrack::Backtrack => {
                        if split.layout == layout {
                            Backtrack::Result(current)
                        } else {
                            Backtrack::Backtrack
                        }
                    }
                },
            },
        }
    }

    fn next_backtrack(&self, from: NodeHandle, current: NodeHandle) -> Backtrack {
        if from == current {
            return Backtrack::Backtrack;
        }

        match &self.nodes[current.idx()] {
            Node::Leaf(_) => Backtrack::Deadend,
            Node::Split(split) => match self.next_backtrack(from, split.right) {
                Backtrack::Result(v) => Backtrack::Result(v),
                Backtrack::Backtrack => Backtrack::Backtrack,
                Backtrack::Deadend => match self.next_backtrack(from, split.left) {
                    Backtrack::Result(v) => Backtrack::Result(v),
                    Backtrack::Deadend => Backtrack::Deadend,
                    Backtrack::Backtrack => {
                        Backtrack::Result(self.deepest(Side::Left, split.right))
                    }
                },
            },
        }
    }

    fn count_after_removal(&self, current: NodeHandle, target: NodeHandle, acc: usize) -> usize {
        assert!(current != target);

        match &self.nodes[current.idx()] {
            Node::Leaf(_) => acc + 1,
            Node::Split(split) => {
                if split.left == target {
                    self.count_after_removal(split.right, target, acc)
                } else if split.right == target {
                    self.count_after_removal(split.left, target, acc)
                } else {
                    self.count_after_removal(split.left, target, acc)
                        + self.count_after_removal(split.right, target, acc)
                        + 1
                }
            }
        }
    }

    fn remove_node(
        &self,
        new_nodes: &mut [Option<Node<V>>],
        zoomed: &mut Option<NodeHandle>,
        new_offset: usize,
        current: NodeHandle,
        target: NodeHandle,
    ) -> usize {
        assert!(current != target);

        if self.zoomed == Some(current) {
            debug_assert!(zoomed.is_none());
            *zoomed = Some(NodeHandle(new_offset as u16));
        }

        match &self.nodes[current.idx()] {
            Node::Leaf(view) => {
                new_nodes[new_offset] = Some(Node::Leaf(Rc::clone(view)));
                1
            }
            Node::Split(split) => {
                if split.left == target {
                    return self.remove_node(new_nodes, zoomed, new_offset, split.right, target);
                }
                if split.right == target {
                    return self.remove_node(new_nodes, zoomed, new_offset, split.left, target);
                }

                let left = self.remove_node(new_nodes, zoomed, new_offset + 1, split.left, target);
                debug_assert!(left != 0);
                let right = self.remove_node(
                    new_nodes,
                    zoomed,
                    new_offset + 1 + left,
                    split.right,
                    target,
                );
                debug_assert!(right != 0);
                new_nodes[new_offset] = Some(Node::Split(SplitNode {
                    layout: split.layout,
                    ratio: split.ratio,
                    left: NodeHandle((new_offset + 1) as u16),
                    right: NodeHandle((new_offset + 1 + left) as u16),
                }));
                left + right + 1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestView {
        label: String,
    }

    fn view(label: &str) -> Rc<TestView> {
        Rc::new(TestView {
            label: label.to_owned(),
        })
    }

    fn handle_for_label(tree: &SplitTree<TestView>, label: &str) -> NodeHandle {
        tree.iterator()
            .find(|entry| entry.view.label == label)
            .map(|entry| entry.handle)
            .unwrap_or_else(|| panic!("label {label} not found"))
    }

    #[test]
    fn is_split() {
        let empty = SplitTree::<TestView>::empty();
        assert!(!empty.is_split());

        let single = SplitTree::new(view("A"));
        assert!(!single.is_split());

        let split = single.split(
            NodeHandle::ROOT,
            SplitDirection::Right,
            0.5,
            &SplitTree::new(view("B")),
        );
        assert!(split.is_split());
    }

    #[test]
    fn horizontal_split_and_remove() {
        let t1 = SplitTree::new(view("A"));
        let t2 = SplitTree::new(view("B"));
        let split = t1.split(NodeHandle::ROOT, SplitDirection::Right, 0.5, &t2);

        let labels: Vec<_> = split
            .iterator()
            .map(|entry| entry.view.label.clone())
            .collect();
        // Flat node storage places the inserted subtree before the moved leaf.
        assert_eq!(labels, vec!["B", "A"]);

        let removed = split.remove(handle_for_label(&split, "A"));
        let labels: Vec<_> = removed
            .iterator()
            .map(|entry| entry.view.label.clone())
            .collect();
        assert_eq!(labels, vec!["B"]);
    }

    #[test]
    fn previous_and_next() {
        let mut tree = SplitTree::new(view("A"));
        tree = tree.split(
            NodeHandle::ROOT,
            SplitDirection::Right,
            0.5,
            &SplitTree::new(view("B")),
        );
        tree = tree.split(
            handle_for_label(&tree, "B"),
            SplitDirection::Right,
            0.5,
            &SplitTree::new(view("C")),
        );
        tree = tree.split(
            handle_for_label(&tree, "C"),
            SplitDirection::Right,
            0.5,
            &SplitTree::new(view("D")),
        );

        let mut current = 'A';
        while current != 'D' {
            let handle = handle_for_label(&tree, &current.to_string());
            let next = tree.next(handle).expect("next handle");
            let next_label = match &tree.nodes()[next.idx()] {
                Node::Leaf(v) => v.label.clone(),
                Node::Split(_) => unreachable!(),
            };
            assert_eq!(next_label, ((current as u8 + 1) as char).to_string());
            current = (current as u8 + 1) as char;
        }

        let mut current = 'D';
        while current != 'A' {
            let handle = handle_for_label(&tree, &current.to_string());
            let previous = tree.previous(handle).expect("previous handle");
            let previous_label = match &tree.nodes()[previous.idx()] {
                Node::Leaf(v) => v.label.clone(),
                Node::Split(_) => unreachable!(),
            };
            assert_eq!(previous_label, ((current as u8 - 1) as char).to_string());
            current = (current as u8 - 1) as char;
        }
    }

    #[test]
    fn zoom_and_remove() {
        let split = SplitTree::new(view("A")).split(
            NodeHandle::ROOT,
            SplitDirection::Right,
            0.5,
            &SplitTree::new(view("B")),
        );

        let mut zoomed = split.clone_tree();
        zoomed.zoom(Some(handle_for_label(&zoomed, "A")));
        assert_eq!(zoomed.zoomed(), Some(handle_for_label(&zoomed, "A")));

        let removed = zoomed.remove(handle_for_label(&zoomed, "A"));
        assert!(removed.zoomed().is_none());
        assert_eq!(
            removed
                .iterator()
                .map(|entry| entry.view.label.clone())
                .collect::<Vec<_>>(),
            vec!["B"]
        );
    }

    fn four_pane_spatial_layout() -> SplitTree<TestView> {
        let split_ab = SplitTree::new(view("A")).split(
            NodeHandle::ROOT,
            SplitDirection::Right,
            0.5,
            &SplitTree::new(view("B")),
        );
        let split_ac = split_ab.split(
            handle_for_label(&split_ab, "A"),
            SplitDirection::Down,
            0.8,
            &SplitTree::new(view("C")),
        );
        split_ac.split(
            handle_for_label(&split_ab, "B"),
            SplitDirection::Down,
            0.3,
            &SplitTree::new(view("D")),
        )
    }

    fn label_at(tree: &SplitTree<TestView>, handle: NodeHandle) -> String {
        match &tree.nodes()[handle.idx()] {
            Node::Leaf(v) => v.label.clone(),
            Node::Split(_) => unreachable!(),
        }
    }

    #[test]
    fn spatial_goto() {
        let split = four_pane_spatial_layout();

        let c = handle_for_label(&split, "C");
        let target = split
            .goto(c, Goto::Spatial(SpatialDirection::Right))
            .expect("C right");
        assert_eq!(label_at(&split, target), "D");

        let d = handle_for_label(&split, "D");
        let target = split
            .goto(d, Goto::Spatial(SpatialDirection::Left))
            .expect("D left");
        assert_eq!(label_at(&split, target), "A");

        let a = handle_for_label(&split, "A");
        let target = split
            .goto(a, Goto::Spatial(SpatialDirection::Left))
            .expect("A left wrapped");
        assert_eq!(label_at(&split, target), "B");

        let b = handle_for_label(&split, "B");
        let target = split
            .goto(b, Goto::Spatial(SpatialDirection::Right))
            .expect("B right wrapped");
        assert_eq!(label_at(&split, target), "A");

        let target = split
            .goto(c, Goto::Spatial(SpatialDirection::Down))
            .expect("C down wrapped");
        assert_eq!(label_at(&split, target), "A");
    }

    #[test]
    fn goto_wrapped_linear() {
        let mut tree = SplitTree::new(view("A"));
        tree = tree.split(
            NodeHandle::ROOT,
            SplitDirection::Right,
            0.5,
            &SplitTree::new(view("B")),
        );
        tree = tree.split(
            handle_for_label(&tree, "B"),
            SplitDirection::Right,
            0.5,
            &SplitTree::new(view("C")),
        );

        let a = handle_for_label(&tree, "A");
        assert_eq!(label_at(&tree, tree.previous_wrapped(a)), "C");
        let c = handle_for_label(&tree, "C");
        assert_eq!(label_at(&tree, tree.next_wrapped(c)), "A");
    }

    #[test]
    fn spatial_normalized_root_slot() {
        let split = SplitTree::new(view("A")).split(
            NodeHandle::ROOT,
            SplitDirection::Right,
            0.5,
            &SplitTree::new(view("B")),
        );
        let sp = split.spatial();
        assert_eq!(sp.slots.len(), split.nodes().len());
        let root = sp.slots[0];
        assert!((root.x - 0.0).abs() < f32::EPSILON);
        assert!((root.y - 0.0).abs() < f32::EPSILON);
        assert!((root.width - 1.0).abs() < f32::EPSILON);
        assert!((root.height - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn resize_horizontal() {
        let split = SplitTree::new(view("A")).split(
            NodeHandle::ROOT,
            SplitDirection::Right,
            0.5,
            &SplitTree::new(view("B")),
        );

        let b = handle_for_label(&split, "B");
        let resized = split.resize(b, SplitLayout::Horizontal, 0.25);
        let ratio = match &resized.nodes()[0] {
            Node::Split(s) => s.ratio,
            Node::Leaf(_) => unreachable!(),
        };
        assert!((ratio - 0.75).abs() < 1e-4);

        let sp = resized.spatial();
        let a_slot = sp.slots[handle_for_label(&resized, "A").idx()];
        assert!((a_slot.width - 0.75).abs() < 1e-4);

        let shrunk = split.resize(b, SplitLayout::Horizontal, -0.25);
        let ratio = match &shrunk.nodes()[0] {
            Node::Split(s) => s.ratio,
            Node::Leaf(_) => unreachable!(),
        };
        assert!((ratio - 0.25).abs() < 1e-4);
    }

    fn inner_bc_split_ratio(tree: &SplitTree<TestView>) -> f32 {
        for node in tree.nodes() {
            if let Node::Split(split) = node {
                let (left_label, right_label) = match (
                    &tree.nodes()[split.left.idx()],
                    &tree.nodes()[split.right.idx()],
                ) {
                    (Node::Leaf(l), Node::Leaf(r)) => (l.label.as_str(), r.label.as_str()),
                    _ => continue,
                };
                if (left_label == "C" && right_label == "B")
                    || (left_label == "B" && right_label == "C")
                {
                    return split.ratio;
                }
            }
        }
        panic!("B/C split not found");
    }

    #[test]
    fn resize_nested_vertical() {
        let split_ab = SplitTree::new(view("A")).split(
            NodeHandle::ROOT,
            SplitDirection::Down,
            0.5,
            &SplitTree::new(view("B")),
        );
        let split = split_ab.split(
            handle_for_label(&split_ab, "B"),
            SplitDirection::Down,
            0.5,
            &SplitTree::new(view("C")),
        );

        let b = handle_for_label(&split, "B");
        let resized = split.resize(b, SplitLayout::Vertical, 0.125);
        assert!((inner_bc_split_ratio(&resized) - 0.75).abs() < 1e-3);

        let shrunk = split.resize(b, SplitLayout::Vertical, -0.0833);
        assert!((inner_bc_split_ratio(&shrunk) - 0.3334).abs() < 1e-2);
    }

    #[test]
    fn equalize_four_pane() {
        let split = four_pane_spatial_layout();
        let equal = split.equalize();
        let sp = equal.spatial();

        for label in ["A", "B", "C", "D"] {
            let slot = sp.slots[handle_for_label(&equal, label).idx()];
            assert!((slot.width - 0.5).abs() < 1e-4, "{label} width");
            assert!((slot.height - 0.5).abs() < 1e-4, "{label} height");
        }
    }

    #[test]
    fn resize_empty_is_noop() {
        let empty = SplitTree::<TestView>::empty();
        let resized = empty.resize(NodeHandle::ROOT, SplitLayout::Horizontal, 0.1);
        assert!(resized.is_empty());
    }
}
