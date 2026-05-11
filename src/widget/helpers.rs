/// Creates a [`Stack`] with the given children.
///
/// [`Stack`]: crate::Stack
#[macro_export]
macro_rules! glass_stack {
    () => (
        $crate::GlassStack::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::GlassStack::with_children([$($crate::widget::InnerContent::from($x)),+])
    );
}
