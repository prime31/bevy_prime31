use bevy::{
    prelude::{App, EventReader, Local, Plugin, ResMut},
    time::Time,
};

/// fire of a Stop event to fully freeze time for the duration or a Slow event to slow time to the passed in value.
/// It will be returned to 1.0 slowly.
pub enum TimeScaleModificationEvent {
    Stop(f32),
    Slow(f32),
}

#[derive(Debug)]
struct TimeStopState {
    elapsed: f32,
    stop_duration: f32,
    time_scale: f32,
}

impl Default for TimeStopState {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            stop_duration: 0.0,
            time_scale: 1.0,
        }
    }
}

/// send a TimeStopEvent with the desired amount of time to stop time for and Time.relative_speed will be 0 for that duration
pub struct TimeManagerPlugin;

impl Plugin for TimeManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TimeScaleModificationEvent>().add_system(update_time);
    }
}

fn update_time(mut events: EventReader<TimeScaleModificationEvent>, mut time: ResMut<Time>, mut state: Local<TimeStopState>) {
    for evt in events.iter() {
        match evt {
            TimeScaleModificationEvent::Stop(length) => if *length > state.stop_duration {
                state.stop_duration = *length;
                state.elapsed = 0.0;
                time.set_relative_speed(0.0);
            },
            TimeScaleModificationEvent::Slow(scale) => state.time_scale = *scale,
        }
    }

    if state.elapsed < state.stop_duration {
        state.elapsed += time.raw_delta_seconds();
        if state.elapsed >= state.stop_duration {
            time.set_relative_speed(1.0);
            *state = TimeStopState::default();
        }
    }

    if state.time_scale < 1.0 {
        if 1.0 - state.time_scale <= 0.02 {
            state.time_scale = 1.0;
        } else {
            state.time_scale += (1.0 - state.time_scale) * 0.02;
        }

        time.set_relative_speed(state.time_scale);
    }
}
