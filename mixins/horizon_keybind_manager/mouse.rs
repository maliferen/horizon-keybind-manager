fn mouse(
    ui: &mut OptionsUi,
    settings: &mut Settings,
    x: f32,
    y: f32,
    size: (f32, f32),
    framebuffer: [i32; 2],
    drag: bool,
    right: bool,
) -> core::ops::ControlFlow<Vec<Event>> {
    if !drag
        && !right
        && keybind_manager_screen::can_activate(ui, settings)
        && let Some(action) = keybind_manager_screen::action_at(ui, settings, (x, y), size)
    {
        ui.size = size;
        ui.framebuffer = framebuffer;
        ui.focus = Some(FocusId::Act(action.clone()));
        ui.focus_from_keyboard = false;
        core::ops::ControlFlow::Break(keybind_manager_screen::activate(ui, settings, action))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
