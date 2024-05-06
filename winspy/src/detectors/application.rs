use chrono::{
    prelude::{DateTime, Utc},
    TimeDelta,
};
use serde::{Deserialize, Serialize};

use super::{DetectedEvent, EventDetector, EventTranscriptReadOnlyView, ProcessedEvent};
use crate::models::persisted_event::{PersistedEvent, PersistedEventPayload};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ApplicationClosedInner {
    executable_name: String,
    executable_sha1_hash: Option<String>,
    opened_at: DateTime<Utc>,
    closed_at: DateTime<Utc>,
    focus_duration_in_seconds: f64,
    user_active_duration_in_seconds: f64,
    number_of_focus_lost_events: u64,
    window_height: u64,
    window_width: u64,
    seconds_of_mouse_input: f64,
    seconds_of_keyboard_input: f64,
    seconds_of_any_user_input: f64,
    seconds_of_audio_recorded: f64,
    seconds_of_audio_played: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ApplicationEventType {
    #[serde(rename = "battery_percentage_change")]
    ApplicationClosed(ApplicationClosedInner),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplicationEvent {
    pub r#type: ApplicationEventType,
}

impl ApplicationEvent {
    pub fn application_closed(info: ApplicationClosedInner) -> Self {
        Self {
            r#type: ApplicationEventType::ApplicationClosed(info),
        }
    }
}

impl From<ApplicationEvent> for DetectedEvent {
    fn from(value: ApplicationEvent) -> Self {
        Self::ApplicationEvent(value)
    }
}

pub struct ApplicationEventDetector;

impl ApplicationEventDetector {
    pub fn new() -> Self {
        Self
    }
}

const APP_INTERACTIVITY_SUMMARY_EVENT_NAME: &str = "Win32kTraceLogging.AppInteractivitySummary";

/// The number of nanoseconds in a millisecond.
const NANOS_PER_MILLISECOND: u32 = 1_000_000;

impl EventDetector for ApplicationEventDetector {
    fn process_event(
        &mut self,
        event: &PersistedEvent,
        _context: &EventTranscriptReadOnlyView,
    ) -> Option<Vec<ProcessedEvent>> {
        if event.event_name().ne(APP_INTERACTIVITY_SUMMARY_EVENT_NAME) {
            return None;
        }

        let PersistedEventPayload::Parsed { payload } = event.payload() else {
            return None;
        };

        let Some(payload) = payload.as_object() else {
            return None;
        };

        let Some(event_time) = payload.get("time").and_then(|field| field.as_str()) else {
            return None;
        };

        let Some(data_table) = payload.get("data").and_then(|field| field.as_object()) else {
            return None;
        };

        // Extract relevant fields from the `data` table.

        let (executable_name, executable_sha1_hash) = {
            let Some(raw_app_id) = data_table.get("AppId").and_then(|field| field.as_str()) else {
                return None;
            };

            let app_id_split = raw_app_id.split('!').collect::<Vec<_>>();

            if app_id_split.len() < 2 {
                return None;
            }

            if app_id_split.len() == 2 {
                let Some(raw_app_version) = data_table
                    .get("AppVersion")
                    .and_then(|field| field.as_str())
                else {
                    return None;
                };

                let Some(alternative_executable_name) = raw_app_version.split('!').last() else {
                    return None;
                };

                (alternative_executable_name.to_string(), None)
            } else {
                // PANIC SAFETY: We just checked that length is not smaller than 2.
                let exeutable_sha1_hash = app_id_split.get(app_id_split.len() - 2).unwrap();

                // PANIC SAFETY: We just checked that length is not smaller than 2.
                let executable_name = app_id_split.last().unwrap();

                (
                    executable_name.to_string(),
                    Some(exeutable_sha1_hash.to_string()),
                )
            }
        };

        let (opened_at, closed_at) = {
            let Ok(event_time) = DateTime::parse_from_rfc3339(event_time) else {
                return None;
            };

            let event_time_utc = event_time.to_utc();

            let Some(since_first_interactivity_ms) = data_table
                .get("SinceFirstInteractivityMS")
                .and_then(|field| field.as_i64())
            else {
                return None;
            };

            let since_first_interactivity_seconds_part = since_first_interactivity_ms / 1000;
            let since_first_interactivity_nanoseconds_part = (since_first_interactivity_ms
                - (since_first_interactivity_seconds_part * 1000))
                as u32
                / NANOS_PER_MILLISECOND;

            let Some(since_first_interactivity_delta) = TimeDelta::new(
                since_first_interactivity_seconds_part,
                since_first_interactivity_nanoseconds_part,
            ) else {
                return None;
            };

            let Some(opened_at) = event_time_utc.checked_sub_signed(since_first_interactivity_delta)
            else {
                return None;
            };

            (opened_at, event_time_utc)
        };

        let focus_duration_in_seconds = {
            let Some(focus_duration_ms) = data_table
                .get("InFocusDurationMS")
                .and_then(|field| field.as_i64())
            else {
                return None;
            };

            (focus_duration_ms as f64) / 1000f64
        };

        let user_active_duration_in_seconds = {
            let Some(active_duration_ms) = data_table
                .get("UserActiveDurationMS")
                .and_then(|field| field.as_i64())
            else {
                return None;
            };

            (active_duration_ms as f64) / 1000f64
        };

        let number_of_focus_lost_events = {
            let Some(focus_lost_times) = data_table
                .get("FocusLostCount")
                .and_then(|field| field.as_i64())
            else {
                return None;
            };

            let Ok(focus_lost_times) = u64::try_from(focus_lost_times) else {
                return None;
            };

            focus_lost_times
        };

        let (window_height, window_width) = {
            let Some(window_width) = data_table
                .get("WindowWidth")
                .and_then(|field| field.as_i64())
            else {
                return None;
            };

            let Ok(window_width) = u64::try_from(window_width) else {
                return None;
            };

            let Some(window_height) = data_table
                .get("WindowHeight")
                .and_then(|field| field.as_i64())
            else {
                return None;
            };

            let Ok(window_height) = u64::try_from(window_height) else {
                return None;
            };

            (window_height, window_width)
        };

        let (seconds_of_any_user_input, seconds_of_mouse_input, seconds_of_keyboard_input) = {
            let Some(total_input_seconds) =
                data_table.get("InputSec").and_then(|field| field.as_i64())
            else {
                return None;
            };

            let Some(keyboard_input_seconds) = data_table
                .get("KeyboardInputSec")
                .and_then(|field| field.as_i64())
            else {
                return None;
            };

            let Some(mouse_input_seconds) = data_table
                .get("MouseInputSec")
                .and_then(|field| field.as_i64())
            else {
                return None;
            };

            (
                total_input_seconds as f64,
                mouse_input_seconds as f64,
                keyboard_input_seconds as f64,
            )
        };

        let (seconds_of_audio_recorded, seconds_of_audio_played) = {
            let Some(audio_recorded_ms) =
                data_table.get("AudioInMS").and_then(|field| field.as_i64())
            else {
                return None;
            };

            let Some(audio_played_ms) = data_table
                .get("AudioOutMS")
                .and_then(|field| field.as_i64())
            else {
                return None;
            };

            (
                (audio_recorded_ms as f64) / 1000f64,
                (audio_played_ms as f64) / 1000f64,
            )
        };

        Some(vec![ProcessedEvent::new_with_random_id(
            event.timestamp().to_owned(),
            ApplicationEvent::application_closed(ApplicationClosedInner {
                executable_name,
                executable_sha1_hash,
                opened_at,
                closed_at,
                focus_duration_in_seconds,
                user_active_duration_in_seconds,
                number_of_focus_lost_events,
                window_height,
                window_width,
                seconds_of_mouse_input,
                seconds_of_keyboard_input,
                seconds_of_any_user_input,
                seconds_of_audio_recorded,
                seconds_of_audio_played,
            }),
        )])
    }
}
