fn keyboard(
    ui: &mut OptionsUi,
    settings: &mut Settings,
    key: winit::keyboard::KeyCode,
) -> core::ops::ControlFlow<Vec<Event>> {
    use winit::keyboard::KeyCode as K;
    if matches!(key, K::Enter | K::NumpadEnter | K::Space)
        && keybind_manager_screen::can_activate(ui, settings)
        && let Some(FocusId::Act(action)) = ui.focus.clone()
        && keybind_manager_screen::is_manager_action(&action)
    {
        core::ops::ControlFlow::Break(keybind_manager_screen::activate(ui, settings, action))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
