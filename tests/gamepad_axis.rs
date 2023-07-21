use bevy::input::gamepad::{
    GamepadAxisChangedEvent, GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo,
};
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::axislike::DualAxisData;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, Debug, Reflect)]
enum ButtonlikeTestAction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Actionlike, Clone, Copy, Debug, Reflect)]
enum AxislikeTestAction {
    X,
    Y,
    XY,
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<ButtonlikeTestAction>::default())
        .add_plugins(InputManagerPlugin::<AxislikeTestAction>::default())
        .init_resource::<ActionState<ButtonlikeTestAction>>()
        .init_resource::<ActionState<AxislikeTestAction>>();

    // WARNING: you MUST register your gamepad during tests, or all gamepad input mocking will fail
    let mut gamepad_events = app.world.resource_mut::<Events<GamepadEvent>>();
    gamepad_events.send(GamepadEvent::Connection(GamepadConnectionEvent {
        // This MUST be consistent with any other mocked events
        gamepad: Gamepad { id: 1 },
        connection: GamepadConnection::Connected(GamepadInfo {
            name: "TestController".into(),
        }),
    }));

    // Ensure that the gamepad is picked up by the appropriate system
    app.update();
    // Ensure that the connection event is flushed through
    app.update();

    app
}

#[test]
fn raw_gamepad_axis_events() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        GamepadAxisType::RightStickX,
        ButtonlikeTestAction::Up,
    )]));

    let mut events = app.world.resource_mut::<Events<GamepadEvent>>();
    events.send(GamepadEvent::Axis(GamepadAxisChangedEvent {
        gamepad: Gamepad { id: 1 },
        axis_type: GamepadAxisType::RightStickX,
        value: 1.0,
    }));

    app.update();
    let action_state = app.world.resource::<ActionState<ButtonlikeTestAction>>();
    assert!(action_state.pressed(ButtonlikeTestAction::Up));
}

#[test]
fn game_pad_single_axis() {
    let mut app = test_app();
    let mut input_map = InputMap::new([
        (GamepadAxisType::LeftStickX, AxislikeTestAction::X),
        (GamepadAxisType::LeftStickY, AxislikeTestAction::Y),
    ]);
    input_map.insert(
        GamepadAxis::new(Gamepad { id: 1 }, GamepadAxisType::LeftStickX),
        AxislikeTestAction::X,
    );
    // input_map.set_gamepad(Gamepad { id: 1 });
    app.insert_resource(input_map);

    // +X
    app.world
        .resource_mut::<Events<GamepadAxisChangedEvent>>()
        .send(GamepadAxisChangedEvent {
            gamepad: Gamepad { id: 1 },
            axis_type: GamepadAxisType::LeftStickX,
            value: 1.0,
        });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::X));

    // -X
    app.world
        .resource_mut::<Events<GamepadAxisChangedEvent>>()
        .send(GamepadAxisChangedEvent {
            gamepad: Gamepad { id: 1 },
            axis_type: GamepadAxisType::LeftStickX,
            value: -1.0,
        });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::X));

    // +Y
    app.world
        .resource_mut::<Events<GamepadAxisChangedEvent>>()
        .send(GamepadAxisChangedEvent {
            gamepad: Gamepad { id: 1 },
            axis_type: GamepadAxisType::LeftStickY,
            value: 1.0,
        });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::Y));

    // -Y
    app.world
        .resource_mut::<Events<GamepadAxisChangedEvent>>()
        .send(GamepadAxisChangedEvent {
            gamepad: Gamepad { id: 1 },
            axis_type: GamepadAxisType::LeftStickY,
            value: -1.0,
        });
    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::Y));

    // 0
    app.world
        .resource_mut::<Events<GamepadAxisChangedEvent>>()
        .send(GamepadAxisChangedEvent {
            gamepad: Gamepad { id: 1 },
            axis_type: GamepadAxisType::LeftStickY,
            value: 0.0,
        });

    app.update();
    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(!action_state.pressed(AxislikeTestAction::Y));
}

#[test]
fn game_pad_dual_axis() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        DualAxis::left_stick(),
        AxislikeTestAction::XY,
    )]));

    app.world
        .resource_mut::<Events<GamepadAxisChangedEvent>>()
        .send(GamepadAxisChangedEvent {
            gamepad: Gamepad { id: 1 },
            axis_type: GamepadAxisType::LeftStickX,
            value: 0.8,
        });

    // TODO: Discover and document why two are required here, in 0.10 only one update was needed.
    //       Could be a sign of a system ordering issue.
    app.update();
    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(action_state.value(AxislikeTestAction::XY), 0.8);
    assert!(action_state.pressed(AxislikeTestAction::XY));
    assert_eq!(
        action_state.axis_pair(AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(0.8, 0.0)
    );

    // Test deadzones, assuming the default of 0.1.
    app.world
        .resource_mut::<Events<GamepadAxisChangedEvent>>()
        .send(GamepadAxisChangedEvent {
            gamepad: Gamepad { id: 1 },
            axis_type: GamepadAxisType::LeftStickX,
            value: -0.05,
        });

    // TODO: Discover and document why two are required here, in 0.10 only one update was needed.
    //       Could be a sign of a system ordering issue.
    app.update();
    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert_eq!(action_state.value(AxislikeTestAction::XY), 0.0);
    assert!(action_state.released(AxislikeTestAction::XY));
    assert_eq!(
        action_state.axis_pair(AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(0.0, 0.0)
    );

    // Test that a single axis below the deadzone is filtered out, assuming the
    // default deadzone of 0.1.
    app.world
        .resource_mut::<Events<GamepadAxisChangedEvent>>()
        .send(GamepadAxisChangedEvent {
            gamepad: Gamepad { id: 1 },
            axis_type: GamepadAxisType::LeftStickX,
            value: 0.2,
        });
    app.world
        .resource_mut::<Events<GamepadAxisChangedEvent>>()
        .send(GamepadAxisChangedEvent {
            gamepad: Gamepad { id: 1 },
            axis_type: GamepadAxisType::LeftStickY,
            value: 0.05,
        });

    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();
    assert!(action_state.pressed(AxislikeTestAction::XY));
    assert_eq!(action_state.value(AxislikeTestAction::XY), 0.2);
    assert_eq!(
        action_state.axis_pair(AxislikeTestAction::XY).unwrap(),
        DualAxisData::new(0.2, 0.0)
    );
}

#[test]
fn game_pad_virtualdpad() {
    let mut app = test_app();
    app.insert_resource(InputMap::new([(
        VirtualDPad::dpad(),
        AxislikeTestAction::XY,
    )]));

    app.world
        .resource_mut::<Input<GamepadButton>>()
        .press(GamepadButton {
            gamepad: Gamepad { id: 1 },
            button_type: GamepadButtonType::DPadLeft,
        });
    app.update();

    let action_state = app.world.resource::<ActionState<AxislikeTestAction>>();

    assert!(action_state.pressed(AxislikeTestAction::XY));
    // This should be unit length, because we're working with a VirtualDpad
    assert_eq!(action_state.value(AxislikeTestAction::XY), 1.0);
    assert_eq!(
        action_state.axis_pair(AxislikeTestAction::XY).unwrap(),
        // This should be unit length, because we're working with a VirtualDpad
        DualAxisData::new(-1.0, 0.0)
    );
}
