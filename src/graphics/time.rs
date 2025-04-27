use std::time::Duration;

use bevy::{app::Plugin, prelude::*};
use time::{macros::format_description, OffsetDateTime};

const DELTA_SECOND: u64 = 60;

#[derive(Component)]
#[require(Text)]
pub struct TimeSpan;

/// 报时插件
pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, alert);
    }
}

fn setup(mut commands: Commands) {
    commands.init_resource::<SystemTimer>();
}

fn alert(
    time: Res<Time<Real>>,
    mut state: ResMut<SystemTimer>,
    mut time_alert: Single<&mut Text, With<TimeSpan>>,
) {
    if !state.tick(time.delta()).just_finished() && !time_alert.0.is_empty() {
        return;
    }

    let Ok(now) = OffsetDateTime::now_local() else {
        return;
    };

    let secs = now.unix_timestamp() as u64 % DELTA_SECOND;
    state.set_elapsed(Duration::from_secs(secs));

    if let Ok(now) = now.format(format_description!("[hour]:[minute]")) {
        time_alert.0 = now;
    }
}

#[derive(Resource, Deref, DerefMut)]
struct SystemTimer(Timer);

impl Default for SystemTimer {
    fn default() -> Self {
        let now = OffsetDateTime::now_local().expect("无法取得系统时间");
        let secs = now.unix_timestamp() as u64 % DELTA_SECOND;
        let mut timer = Timer::new(Duration::from_secs(DELTA_SECOND), TimerMode::Repeating);
        timer.set_elapsed(Duration::from_secs(secs));
        Self(timer)
    }
}
