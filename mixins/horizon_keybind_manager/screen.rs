use super::keybind_manager_runtime::{Chord, Search, UNBOUND, next_conflict};
use super::{
    Act, CATEGORIES, Event, FocusId, ListRows, OptionsUi, Page, Settings, Style, Text, WKind,
    Widget, all_mappings, button, inactive, keybinds, lang, text, tr,
};

// Use the native widget focus system with a reserved index. The input hooks
// handle these controls before the options screen dispatches any mod event.
const MANAGER_BUTTON: usize = usize::MAX;
const FILTER: usize = 0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum BindingId {
    Game(&'static str),
    Mod(usize, usize),
}

impl BindingId {
    fn change(self) -> Act {
        match self {
            Self::Game(name) => Act::Key(name),
            Self::Mod(mod_index, key_index) => Act::NativeKey(mod_index, key_index),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScreenState {
    conflicts_only: bool,
    jump: Option<BindingId>,
}

struct Binding<'a> {
    id: BindingId,
    label: Text,
    group: Text,
    chord: Chord<'a>,
    default: bool,
}

fn local(en: &str, ru: &str) -> Text {
    Text::literal(if lang::code() == "ru_ru" { ru } else { en })
}

fn bindings(settings: &Settings) -> Vec<Binding<'_>> {
    let mut game: Vec<_> = all_mappings().collect();
    game.sort_by(|a, b| {
        let category = |name| CATEGORIES.iter().position(|c| *c == name).unwrap_or(99);
        category(a.2)
            .cmp(&category(b.2))
            .then_with(|| lang::get(a.0).cmp(&lang::get(b.0)))
    });
    let mut result: Vec<_> = game
        .into_iter()
        .map(|&(name, _, category)| {
            let default = settings.keys.is_default(name);
            Binding {
                id: BindingId::Game(name),
                label: tr(name),
                group: tr(&format!("key.category.minecraft.{category}")),
                chord: Chord {
                    key: settings.keys.get(name),
                    with_debug: category == "debug"
                        && !matches!(name, "key.debug.modifier" | "key.debug.overlay"),
                    default_game_key: default,
                },
                default,
            }
        })
        .collect();
    let mut mods: Vec<_> = settings.mods.installed.iter().enumerate().collect();
    mods.sort_by(|a, b| a.1.title.cmp(&b.1.title));
    for (mod_index, installed) in mods {
        let Some(page) = &installed.native else {
            continue;
        };
        if installed.problem.is_some() || settings.mods.disabled.contains(&installed.id) {
            continue;
        }
        let chosen = settings.mods.keys.get(&installed.id);
        for (key_index, [name, label, default]) in page.keys.iter().enumerate() {
            let key = chosen.and_then(|keys| keys.get(name)).unwrap_or(default);
            result.push(Binding {
                id: BindingId::Mod(mod_index, key_index),
                label: Text::literal(label),
                group: Text::literal(&installed.title),
                chord: Chord {
                    key: if key.is_empty() { UNBOUND } else { key },
                    with_debug: false,
                    default_game_key: false,
                },
                default: key == default,
            });
        }
    }
    result
}

fn caption(x: f32, y: f32, w: f32, label: Text, centered: bool, color: u32) -> Widget {
    Widget {
        x,
        y,
        w,
        h: 9.0,
        kind: WKind::Text {
            text: label,
            color,
            centered,
        },
        list: false,
        row: 0,
    }
}

fn manager_action(index: usize) -> Act {
    Act::NativeButton(MANAGER_BUTTON, index)
}

pub(super) fn layout(
    ui: &mut OptionsUi,
    settings: &Settings,
    width: f32,
    height: f32,
) -> (Vec<Widget>, f32, f32, f32) {
    let bindings = bindings(settings);
    let chords: Vec<_> = bindings.iter().map(|b| b.chord).collect();
    let state = ui
        .settings_search
        .__horizon_mixins_horizon_keybind_manager_screen_state_mut();
    let conflicts_only = state.conflicts_only;
    let jump = state.jump.take();
    let query = Search::new(&ui.settings_search.value);
    let cx = (width / 2.0).floor();
    ui.settings_search.x = cx - 100.0;
    ui.settings_search.y = 17.0;

    let top = 59.0;
    let bottom = (height - 33.0).max(top);
    let mut filter = local("Conflicts only: Off", "Только конфликты: Выкл");
    if conflicts_only {
        filter = local("Conflicts only: On", "Только конфликты: Вкл").with_color(text::YELLOW);
    }
    let mut out = vec![
        caption(
            0.0,
            4.0,
            width,
            tr(OptionsUi::title(Page::KeyBinds)),
            true,
            u32::MAX,
        ),
        button(cx - 100.0, 35.0, 200.0, filter, manager_action(FILTER)),
    ];
    let list_start = out.len();
    let mut rows = ListRows::default();
    let row_width = (width - 18.0).min(340.0);
    let row_left = cx - row_width / 2.0;
    let reset_x = cx + row_width / 2.0 - 52.0;
    let change_x = reset_x - 5.0 - 75.0;
    let jump_x = change_x - 23.0;
    let mut y = top + 2.0 - ui.scroll;
    let mut group = String::new();
    let mut destination = None;

    for (index, binding) in bindings.iter().enumerate() {
        let name = binding.label.plain();
        let category = binding.group.plain();
        let mut key_label = keybinds::display_name(binding.chord.key);
        let has_conflict = next_conflict(&chords, index).is_some();
        if (conflicts_only && !has_conflict)
            || !query.matches(&name, &category, binding.chord.key, &key_label.plain())
        {
            continue;
        }
        if group != category {
            group = category;
            rows.begin(&out, y);
            let label = binding.group.clone().with_style(&Style {
                bold: Some(true),
                underlined: Some(true),
                ..Style::default()
            });
            out.push(caption(0.0, y + 7.0, width, label, true, u32::MAX));
            y += 20.0;
            rows.end(&mut out, y);
        }
        rows.begin(&out, y);
        let label_width = (jump_x - row_left - 5.0).max(0.0);
        out.push(caption(
            row_left + 2.0,
            y + 6.0,
            label_width,
            binding.label.clone(),
            false,
            u32::MAX,
        ));
        let action = binding.id.change();
        let waiting = match binding.id {
            BindingId::Game(name) => ui.selected_key == Some(name),
            BindingId::Mod(i, k) => ui.selected_native_key == Some((i, k)),
        };
        if waiting {
            key_label = Text::literal("> ")
                .append(key_label)
                .append(" <")
                .with_color(text::YELLOW);
        } else if has_conflict {
            key_label = key_label.with_color(text::YELLOW);
        }
        if has_conflict {
            out.push(button(
                jump_x,
                y,
                20.0,
                Text::literal("!").with_color(text::YELLOW),
                manager_action(index * 2 + 1),
            ));
        }
        if jump == Some(binding.id) {
            destination = Some((y + ui.scroll, action.clone()));
        }
        out.push(button(change_x, y, 75.0, key_label, action));
        let reset_action = match binding.id {
            BindingId::Game(name) => Act::ResetKey(name),
            BindingId::Mod(..) => manager_action(index * 2 + 2),
        };
        let reset = button(reset_x, y, 50.0, tr("controls.reset"), reset_action);
        out.push(if binding.default {
            inactive(reset)
        } else {
            reset
        });
        y += 20.0;
        rows.end(&mut out, y);
    }
    if list_start == out.len() {
        let empty = if conflicts_only && ui.settings_search.value.trim().is_empty() {
            local("No key conflicts", "Конфликтов клавиш нет")
        } else {
            local("No matching key binds", "Подходящих назначений нет")
        };
        out.push(caption(0.0, y + 8.0, width, empty, true, 0xffaaaaaa));
        y += 24.0;
    }
    let previous_scroll = ui.scroll;
    let content = y + previous_scroll - top + 2.0;
    if let Some((row_y, action)) = destination {
        ui.scroll = row_y - top - ((bottom - top - 20.0) * 0.5).max(0.0);
        ui.focus = Some(FocusId::Act(action));
        ui.focus_from_keyboard = true;
        ui.can_change_value = false;
    }
    ui.clamp_scroll(top, bottom, content);
    let offset = previous_scroll - ui.scroll;
    for widget in &mut out[list_start..] {
        widget.y += offset;
        widget.list = true;
    }
    ui.row_bands = rows
        .bands
        .into_iter()
        .map(|[a, b]| [a + offset, b + offset])
        .collect();
    let footer_width = (width / 2.0 - 8.0).min(150.0);
    out.push(button(
        cx - 4.0 - footer_width,
        height - 27.0,
        footer_width,
        tr("controls.resetAll"),
        Act::ResetKeys,
    ));
    out.push(button(
        cx + 4.0,
        height - 27.0,
        footer_width,
        tr("gui.done"),
        Act::Done,
    ));
    (out, top, bottom, content)
}

pub(super) fn enabled(ui: &OptionsUi, settings: &Settings) -> bool {
    ui.page() == Page::KeyBinds
        && !settings
            .mods
            .disabled
            .contains(super::keybind_manager_runtime::ID)
}

pub(super) fn can_activate(ui: &OptionsUi, settings: &Settings) -> bool {
    enabled(ui, settings)
        && ui.resource_error.is_none()
        && ui.selected_key.is_none()
        && ui.selected_pad_action.is_none()
}

pub(super) fn is_manager_action(action: &Act) -> bool {
    matches!(action, Act::NativeButton(MANAGER_BUTTON, _))
}

pub(super) fn action_at(
    ui: &mut OptionsUi,
    settings: &Settings,
    cursor: (f32, f32),
    (width, height): (f32, f32),
) -> Option<Act> {
    let (widgets, top, bottom, _) = layout(ui, settings, width, height);
    widgets.into_iter().find_map(|widget| {
        if !super::ui::hovered(cursor, widget.x, widget.y, widget.w, widget.h)
            || (widget.list && !(top..bottom).contains(&cursor.1))
        {
            return None;
        }
        match widget.kind {
            WKind::Button {
                act, active: true, ..
            } if is_manager_action(&act) => Some(act),
            _ => None,
        }
    })
}

pub(super) fn activate(ui: &mut OptionsUi, settings: &mut Settings, action: Act) -> Vec<Event> {
    let Act::NativeButton(MANAGER_BUTTON, index) = action else {
        return Vec::new();
    };
    ui.settings_search.focused = false;
    ui.can_change_value = false;
    let mut events = vec![Event::ButtonClick];
    if index == FILTER {
        let state = ui
            .settings_search
            .__horizon_mixins_horizon_keybind_manager_screen_state_mut();
        state.conflicts_only = !state.conflicts_only;
        ui.scroll = 0.0;
        return events;
    }
    let bindings = bindings(settings);
    let row = (index - 1) / 2;
    let Some(binding) = bindings.get(row) else {
        return events;
    };
    if index % 2 == 1 {
        let chords: Vec<_> = bindings.iter().map(|b| b.chord).collect();
        if let Some(next) = next_conflict(&chords, row) {
            let display = keybinds::display_name(binding.chord.key).plain();
            ui.settings_search.set_value(&format!("key:{display}"));
            ui.settings_search
                .__horizon_mixins_horizon_keybind_manager_screen_state_mut()
                .jump = Some(bindings[next].id);
        }
    } else if let BindingId::Mod(mod_index, key_index) = binding.id
        && let Some(installed) = settings.mods.installed.get(mod_index)
        && let Some([name, ..]) = installed
            .native
            .as_ref()
            .and_then(|page| page.keys.get(key_index))
        && let Some(keys) = settings.mods.keys.get_mut(&installed.id)
        && keys.remove(name).is_some()
    {
        events.push(Event::Changed);
    }
    events
}

pub(super) fn tooltip(settings: &Settings, widget: &Widget) -> Option<Text> {
    let WKind::Button {
        act: Act::NativeButton(MANAGER_BUTTON, index),
        ..
    } = widget.kind
    else {
        return None;
    };
    if index == FILTER {
        return Some(local(
            "Show actions that use the same key",
            "Показать действия, назначенные на одну клавишу",
        ));
    }
    if index % 2 == 0 {
        return None;
    }
    let bindings = bindings(settings);
    let chords: Vec<_> = bindings.iter().map(|b| b.chord).collect();
    let next = next_conflict(&chords, (index - 1) / 2)?;
    Some(local("Next conflict: ", "Следующий конфликт: ").append(bindings[next].label.clone()))
}
