extern crate gilrs;

use self::gilrs::*;
use std::sync::Mutex;

pub enum Button {
    North,
    East,
    South,
    West,
    BumperLeft,
    BumperRight,
    DUp,
    DRight,
    DDown,
    DLeft,
}

pub enum Axis {
    MoveX,
    MoveY,
    ViewX,
    ViewY,
    TriggerL,
    TriggerR,
}

#[derive(Debug, Default, Copy, Clone)]
struct ControllerState {
    left: (f32, f32),
    right: (f32, f32),
    triggers: (f32, f32),
    bumpers: (bool, bool),
    buttons: (bool, bool, bool, bool),
    dpad: (bool, bool, bool, bool),
}

lazy_static! {
    static ref CONTROLLER: Mutex<ControllerState> =
        Mutex::new(ControllerState::default());
    static ref LAST_CONTROLLER: Mutex<ControllerState> =
        Mutex::new(ControllerState::default());
}

pub fn init() -> Gilrs {
    debug!("Input initialized");
    Gilrs::new().unwrap()
}

pub fn events(g: &mut Gilrs) {
    *LAST_CONTROLLER.lock().unwrap() = *CONTROLLER.lock().unwrap();
    while let Some(Event { event, .. }) = g.next_event() {
        match event {
            EventType::Connected => info!("Controller connected"),
            EventType::Disconnected => info!("Controller disconnected"),
            EventType::ButtonPressed(ev::Button::North, _) => {
                CONTROLLER.lock().unwrap().buttons.0 = true
            }
            EventType::ButtonPressed(ev::Button::East, _) => {
                CONTROLLER.lock().unwrap().buttons.1 = true
            }
            EventType::ButtonPressed(ev::Button::South, _) => {
                CONTROLLER.lock().unwrap().buttons.2 = true
            }
            EventType::ButtonPressed(ev::Button::West, _) => {
                CONTROLLER.lock().unwrap().buttons.3 = true
            }
            EventType::ButtonPressed(ev::Button::LeftTrigger, _) => {
                CONTROLLER.lock().unwrap().bumpers.0 = true
            }
            EventType::ButtonPressed(ev::Button::RightTrigger, _) => {
                CONTROLLER.lock().unwrap().bumpers.1 = true
            }
            EventType::ButtonPressed(ev::Button::DPadUp, _) => {
                CONTROLLER.lock().unwrap().dpad.0 = true
            }
            EventType::ButtonPressed(ev::Button::DPadRight, _) => {
                CONTROLLER.lock().unwrap().dpad.1 = true
            }
            EventType::ButtonPressed(ev::Button::DPadDown, _) => {
                CONTROLLER.lock().unwrap().dpad.2 = true
            }
            EventType::ButtonPressed(ev::Button::DPadLeft, _) => {
                CONTROLLER.lock().unwrap().dpad.3 = true
            }

            EventType::ButtonReleased(ev::Button::North, _) => {
                CONTROLLER.lock().unwrap().buttons.0 = false
            }
            EventType::ButtonReleased(ev::Button::East, _) => {
                CONTROLLER.lock().unwrap().buttons.1 = false
            }
            EventType::ButtonReleased(ev::Button::South, _) => {
                CONTROLLER.lock().unwrap().buttons.2 = false
            }
            EventType::ButtonReleased(ev::Button::West, _) => {
                CONTROLLER.lock().unwrap().buttons.3 = false
            }
            EventType::ButtonReleased(ev::Button::LeftTrigger, _) => {
                CONTROLLER.lock().unwrap().bumpers.0 = false
            }
            EventType::ButtonReleased(ev::Button::RightTrigger, _) => {
                CONTROLLER.lock().unwrap().bumpers.1 = false
            }
            EventType::ButtonReleased(ev::Button::DPadUp, _) => {
                CONTROLLER.lock().unwrap().dpad.0 = false
            }
            EventType::ButtonReleased(ev::Button::DPadRight, _) => {
                CONTROLLER.lock().unwrap().dpad.1 = false
            }
            EventType::ButtonReleased(ev::Button::DPadDown, _) => {
                CONTROLLER.lock().unwrap().dpad.2 = false
            }
            EventType::ButtonReleased(ev::Button::DPadLeft, _) => {
                CONTROLLER.lock().unwrap().dpad.3 = false
            }

            EventType::AxisChanged(ev::Axis::LeftStickX, v, _) => {
                CONTROLLER.lock().unwrap().left.0 = v
            }
            EventType::AxisChanged(ev::Axis::LeftStickY, v, _) => {
                CONTROLLER.lock().unwrap().left.1 = v
            }
            EventType::AxisChanged(ev::Axis::RightStickX, v, _) => {
                CONTROLLER.lock().unwrap().right.0 = v
            }
            EventType::AxisChanged(ev::Axis::RightStickY, v, _) => {
                CONTROLLER.lock().unwrap().right.1 = v
            }

            EventType::ButtonChanged(ev::Button::LeftTrigger2, v, _) => {
                CONTROLLER.lock().unwrap().triggers.0 = v
            }
            EventType::ButtonChanged(ev::Button::RightTrigger2, v, _) => {
                CONTROLLER.lock().unwrap().triggers.1 = v
            }

            e => trace!("Ignored event: {:?}", e),
        }
    }
}

pub fn get_button(button: &Button) -> bool {
    match button {
        Button::North => CONTROLLER.lock().unwrap().buttons.0,
        Button::East => CONTROLLER.lock().unwrap().buttons.1,
        Button::South => CONTROLLER.lock().unwrap().buttons.2,
        Button::West => CONTROLLER.lock().unwrap().buttons.3,
        Button::BumperLeft => CONTROLLER.lock().unwrap().bumpers.0,
        Button::BumperRight => CONTROLLER.lock().unwrap().bumpers.1,
        Button::DUp => CONTROLLER.lock().unwrap().dpad.0,
        Button::DRight => CONTROLLER.lock().unwrap().dpad.1,
        Button::DDown => CONTROLLER.lock().unwrap().dpad.2,
        Button::DLeft => CONTROLLER.lock().unwrap().dpad.3,
    }
}

pub fn get_button_down(button: &Button) -> bool {
    match button {
        Button::North => {
            CONTROLLER.lock().unwrap().buttons.0
                && !LAST_CONTROLLER.lock().unwrap().buttons.0
        }
        Button::East => {
            CONTROLLER.lock().unwrap().buttons.1
                && !LAST_CONTROLLER.lock().unwrap().buttons.1
        }
        Button::South => {
            CONTROLLER.lock().unwrap().buttons.2
                && !LAST_CONTROLLER.lock().unwrap().buttons.2
        }
        Button::West => {
            CONTROLLER.lock().unwrap().buttons.3
                && !LAST_CONTROLLER.lock().unwrap().buttons.3
        }
        Button::BumperLeft => {
            CONTROLLER.lock().unwrap().bumpers.0
                && !LAST_CONTROLLER.lock().unwrap().bumpers.0
        }
        Button::BumperRight => {
            CONTROLLER.lock().unwrap().bumpers.1
                && !LAST_CONTROLLER.lock().unwrap().bumpers.1
        }
        Button::DUp => {
            CONTROLLER.lock().unwrap().dpad.0
                && !LAST_CONTROLLER.lock().unwrap().dpad.0
        }
        Button::DRight => {
            CONTROLLER.lock().unwrap().dpad.1
                && !LAST_CONTROLLER.lock().unwrap().dpad.1
        }
        Button::DDown => {
            CONTROLLER.lock().unwrap().dpad.2
                && !LAST_CONTROLLER.lock().unwrap().dpad.2
        }
        Button::DLeft => {
            CONTROLLER.lock().unwrap().dpad.3
                && !LAST_CONTROLLER.lock().unwrap().dpad.3
        }
    }
}

pub fn get_button_up(button: &Button) -> bool {
    match button {
        Button::North => {
            !CONTROLLER.lock().unwrap().buttons.0
                && LAST_CONTROLLER.lock().unwrap().buttons.0
        }
        Button::East => {
            !CONTROLLER.lock().unwrap().buttons.1
                && LAST_CONTROLLER.lock().unwrap().buttons.1
        }
        Button::South => {
            !CONTROLLER.lock().unwrap().buttons.2
                && LAST_CONTROLLER.lock().unwrap().buttons.2
        }
        Button::West => {
            !CONTROLLER.lock().unwrap().buttons.3
                && LAST_CONTROLLER.lock().unwrap().buttons.3
        }
        Button::BumperLeft => {
            !CONTROLLER.lock().unwrap().bumpers.0
                && LAST_CONTROLLER.lock().unwrap().bumpers.0
        }
        Button::BumperRight => {
            !CONTROLLER.lock().unwrap().bumpers.1
                && LAST_CONTROLLER.lock().unwrap().bumpers.1
        }
        Button::DUp => {
            !CONTROLLER.lock().unwrap().dpad.0
                && LAST_CONTROLLER.lock().unwrap().dpad.0
        }
        Button::DRight => {
            !CONTROLLER.lock().unwrap().dpad.1
                && LAST_CONTROLLER.lock().unwrap().dpad.1
        }
        Button::DDown => {
            !CONTROLLER.lock().unwrap().dpad.2
                && LAST_CONTROLLER.lock().unwrap().dpad.2
        }
        Button::DLeft => {
            !CONTROLLER.lock().unwrap().dpad.3
                && LAST_CONTROLLER.lock().unwrap().dpad.3
        }
    }
}

pub fn get_axis(axis: &Axis) -> f32 {
    match axis {
        Axis::MoveX => CONTROLLER.lock().unwrap().left.0,
        Axis::MoveY => CONTROLLER.lock().unwrap().left.1,
        Axis::ViewX => CONTROLLER.lock().unwrap().right.0,
        Axis::ViewY => CONTROLLER.lock().unwrap().right.1,
        Axis::TriggerL => CONTROLLER.lock().unwrap().triggers.0,
        Axis::TriggerR => CONTROLLER.lock().unwrap().triggers.0,
    }
}
