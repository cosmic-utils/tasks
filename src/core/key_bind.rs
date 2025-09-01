use std::collections::HashMap;

use cosmic::{
    iced::keyboard::Key,
    iced_core::keyboard::key::Named,
    widget::menu::key_bind::{KeyBind, Modifier},
};

use crate::app::actions::Action;

pub fn key_binds() -> HashMap<KeyBind, Action> {
    let mut key_binds = HashMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:ident) => {{
            key_binds.insert(
                KeyBind {
                    modifiers: vec![$(Modifier::$modifier),*],
                    key: $key,
                },
                Action::$action,
            );
        }};
    }

    bind!([Ctrl], Key::Character("n".into()), NewList);
    bind!([Ctrl], Key::Named(Named::Delete), DeleteList);
    bind!([Ctrl], Key::Character("r".into()), RenameList);
    
    bind!([Ctrl], Key::Character("w".into()), WindowClose);
    bind!([Ctrl, Shift], Key::Character("n".into()), WindowNew);
    bind!([Ctrl], Key::Character(",".into()), Settings);
    bind!([Ctrl], Key::Character("i".into()), About);

    key_binds
}
