fn list(mods: &mut Vec<NativeMod>) {
    use horizon_client_render::options_ui::keybind_manager_runtime as manager;
    mods.push(NativeMod {
        id: manager::ID.into(),
        title: format!("{} {}", manager::TITLE, manager::VERSION),
        description: "Search key binds and resolve conflicts with native mods.".into(),
        keys: Vec::new(),
    });
}
