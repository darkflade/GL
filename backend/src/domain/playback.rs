use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackDirection {
    #[default]
    Forward,
    Backward,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MediaEnd {
    Repeat,
    #[default]
    Next,
    Stop,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PlaybackSettings {
    pub shortcut_item_id: Option<uuid::Uuid>,
    pub interval_seconds: u32,
    pub direction: PlaybackDirection,
    pub loop_collection: bool,
    pub skip_audio: bool,
    pub skip_video: bool,
    pub mute_video_with_audio: bool,
    pub audio_end: MediaEnd,
    pub video_end: MediaEnd,
}
impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            shortcut_item_id: None,
            interval_seconds: 5,
            direction: PlaybackDirection::Forward,
            loop_collection: true,
            skip_audio: true,
            skip_video: false,
            mute_video_with_audio: true,
            audio_end: MediaEnd::Next,
            video_end: MediaEnd::Next,
        }
    }
}
#[derive(Serialize)]
pub struct PlaybackPreferences {
    pub item_id: Option<uuid::Uuid>,
    pub global: PlaybackSettings,
    pub collection: Option<PlaybackSettings>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_saved_settings_inherit_defaults_and_reject_unknown_values() {
        let settings: PlaybackSettings = serde_json::from_str(r#"{"skip_video":true}"#).unwrap();
        assert!(settings.skip_video && settings.skip_audio);
        assert_eq!(settings.interval_seconds, 5);
        assert!(settings.shortcut_item_id.is_none());
        let id = uuid::Uuid::now_v7();
        let saved: PlaybackSettings =
            serde_json::from_value(serde_json::json!({"shortcut_item_id":id})).unwrap();
        assert_eq!(saved.shortcut_item_id, Some(id));
        let restored: PlaybackSettings =
            serde_json::from_value(serde_json::to_value(saved).unwrap()).unwrap();
        assert_eq!(restored.shortcut_item_id, Some(id));
        assert!(serde_json::from_str::<PlaybackSettings>(r#"{"direction":"random"}"#).is_err());
        assert!(serde_json::from_str::<PlaybackSettings>(r#"{"audio_end":"unknown"}"#).is_err());
        assert!(serde_json::from_str::<PlaybackSettings>(r#"{"interval_seconds":-1}"#).is_err());
        assert!(serde_json::from_str::<PlaybackSettings>(r#"{"skip_audo":true}"#).is_err());
    }
}
