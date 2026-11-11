//! Actions triggered from native menus, identified by menu item ids.

use tray_icon::menu::MenuId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    OpenPreferences,
    Reload,
    OpenProfile,
    OpenLatestRelease,
    About,
    Quit,
    CloseWindow,
    Edit(EditAction),
}

/// Text editing actions from the macOS Edit menu. The menu's key equivalents swallow the
/// keystrokes, so these are forwarded to egui as the equivalent input events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditAction {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
}

const ALL: [Command; 13] = [
    Command::OpenPreferences,
    Command::Reload,
    Command::OpenProfile,
    Command::OpenLatestRelease,
    Command::About,
    Command::Quit,
    Command::CloseWindow,
    Command::Edit(EditAction::Undo),
    Command::Edit(EditAction::Redo),
    Command::Edit(EditAction::Cut),
    Command::Edit(EditAction::Copy),
    Command::Edit(EditAction::Paste),
    Command::Edit(EditAction::SelectAll),
];

impl Command {
    pub fn id(self) -> MenuId {
        MenuId::new(format!("{self:?}"))
    }

    pub fn from_id(id: &MenuId) -> Option<Self> {
        ALL.into_iter().find(|command| command.id() == *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip() {
        for command in ALL {
            assert_eq!(Command::from_id(&command.id()), Some(command));
        }
        assert_eq!(Command::from_id(&MenuId::new("unknown")), None);
    }
}
