/// App layout mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLayout {
    /// Render panes as a vertical stack.
    VerticalStack,
    /// Render panes in a split-view layout.
    SplitView,
}
