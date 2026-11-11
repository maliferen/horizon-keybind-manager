fn reset_keybind_manager_page(ui: &mut OptionsUi, page: Page) {
    if page == Page::KeyBinds {
        *ui.settings_search
            .__horizon_mixins_horizon_keybind_manager_screen_state_mut() = Default::default();
    }
}
