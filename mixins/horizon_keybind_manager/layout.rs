fn layout(
    ui: &mut OptionsUi,
    settings: &Settings,
    width: f32,
    height: f32,
) -> core::ops::ControlFlow<(Vec<Widget>, f32, f32, f32)> {
    if keybind_manager_screen::enabled(ui, settings) {
        core::ops::ControlFlow::Break(keybind_manager_screen::layout(ui, settings, width, height))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
