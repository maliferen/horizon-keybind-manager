fn tooltip(settings: &Settings, widget: &Widget) -> core::ops::ControlFlow<Option<Text>> {
    match keybind_manager_screen::tooltip(settings, widget) {
        Some(text) => core::ops::ControlFlow::Break(Some(text)),
        None => core::ops::ControlFlow::Continue(()),
    }
}
