use std::sync::Arc;

use evie_core::{
    trigger::{Modes, Trigger, TriggerMap},
    Mode,
};

use crate::KeyAction;
use crate::Named;

use evie_core::{BufferAction::*, CoreAction::*};
use Trigger::*;

pub fn modes() -> Modes<KeyAction> {
    Modes::new(
        TriggerMap::from([(KeyAction::Letter('i'), End(SetMode(Mode::Insert).into()))]),
        TriggerMap::from((
            [],
            call(|ka| match ka {
                &KeyAction::Letter(s) => Some(Trigger::End(Append(s.into()).into())),
                KeyAction::Named(Named::Enter) => Some(Trigger::End(Append('\n'.into()).into())),
                _ => None,
            }),
        )),
        TriggerMap::from([]),
        TriggerMap::from([]),
        TriggerMap::from([]),
        TriggerMap::from([]),
        TriggerMap::from((
            [],
            call(|ka| match ka {
                KeyAction::Named(Named::Escape) => Some(End(SetMode(Mode::Normal).into())),
                _ => None,
            }),
        )),
    )
}

fn call(
    call: impl Fn(&KeyAction) -> Option<Trigger<KeyAction>> + 'static,
) -> Arc<dyn Fn(&KeyAction) -> Option<Trigger<KeyAction>>> {
    Arc::new(move |ka| call(ka))
}
