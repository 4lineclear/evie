// use dashmap::DashMap;

use std::sync::Arc;

use evie_core::{Mode, Modes, Trigger, TriggerMap};

use crate::KeyAction;

macro_rules! modes {
    ($(
        $e:expr
    ) ,*) => {
        Modes::new($(
            TriggerMap::from($e),
        )*)
    };
}

use evie_core::BufferAction::*;
use evie_core::CoreAction::*;
use evie_core::Trigger::*;

pub fn modes() -> Modes<KeyAction> {
    modes!(
        [(KeyAction::Letter('i'), End(SetMode(Mode::Insert).into()))],
        (
            [],
            call(|ka| match ka {
                &KeyAction::Letter(s) => Some(Trigger::End(Append(s.into()).into())),
                KeyAction::Escape => Some(End(SetMode(Mode::Normal).into())),
                KeyAction::Enter => Some(Trigger::End(Append('\n'.into()).into())),
            })
        ),
        [],
        [],
        [],
        [],
        []
    )
    // Modes::new(
    //     [(KeyAction::Letter('i'), End(SetMode(Mode::Insert).into()))], //
    //     modes!([]).with_fallback(Arc::new(|ka| match ka {
    //         &KeyAction::Letter(s) => Some(Trigger::End(Append(s.into()).into())),
    //         KeyAction::Escape => Some(End(SetMode(Mode::Normal).into())),
    //         KeyAction::Enter => Some(Trigger::End(Append('\n'.into()).into())),
    //     })),
    //     modes!(),
    //     modes!(),
    //     modes!(),
    //     modes!(),
    //     modes!(),
    // )
}

fn call(
    call: impl Fn(&KeyAction) -> Option<Trigger<KeyAction>> + 'static,
) -> Arc<dyn Fn(&KeyAction) -> Option<Trigger<KeyAction>>> {
    Arc::new(move |ka| call(ka))
}
