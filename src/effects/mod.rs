//! Custom JSON lighting effects and profile engine.

pub mod player;
pub mod schema;
pub mod templates;

pub use player::{load_effect_from_file, play_custom_effect};
pub use schema::{ColorRGB, CustomEffect, EffectKeyframe, ZoneState};
pub use templates::get_template_json;
