use lavalink_rs::client::LavalinkClient;
use lavalink_rs::model::track::TrackData;
use lavalink_rs::player_context::QueueRef;
use serenity::model::Colour;
use serenity::prelude::TypeMapKey;
use serenity::{builder::CreateEmbed, client::Context, model::id::GuildId};
use std::{cmp::Ordering, time::Duration};
use std::{
    error::Error,
    fmt::{Display, Write},
};
use tracing::log::warn;

//Modules
pub mod commands;
pub mod events;

// Constants
pub const EMBED_COLOR: Colour = Colour::ORANGE;
pub const MAX_DESCRIPTION_SIZE: usize = 4094;

// Structs
pub struct Lavalink;

impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}

pub type FerrisResponse = Result<CreateEmbed, Box<dyn Error + Sync + Send>>;

/// Struct for errors that should be returned as a message on Discord
#[derive(Debug)]
pub enum FerrisError {
    LavalinkError,
    TrackNotFoundError,
    QueueEmptyError,
    PositionError,
    AlreadyOnCallError,
    NotOnCallError,
    GuildError,
    MissingArguments,
}
impl Error for FerrisError {}
impl Display for FerrisError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FerrisError::LavalinkError => write!(f, "Someting went wrong with Lavalink"),
            FerrisError::QueueEmptyError => write!(f, "Nothing is in queue"),
            FerrisError::PositionError => write!(f, "Could not parse position"),
            FerrisError::AlreadyOnCallError => write!(f, "Already on a voice channel"),
            FerrisError::NotOnCallError => write!(f, "Not on a voice channel"),
            FerrisError::GuildError => write!(f, "Could not get guild infromation"),
            FerrisError::MissingArguments => write!(f, "Missing arguments"),
            FerrisError::TrackNotFoundError => write!(f, "Track could not be found"),
        }
    }
}

/// Looping track type. This is a newtype for lavalinks Track type as we need a seperate type for TypeMap in Node.
/// This keeps track of which track is looping since we need to requeue it in track end event handler.
#[derive(Clone, Debug)]
pub struct LoopingTrack(TrackData);

/// Allows building a response message in embedded Discord message format
///
/// # Examples
///
/// ```
/// use dj_ferris::Response;
///
/// let response_description_only = Response::new().title("Description text").build();
/// let response_title = Response::new().title("Title text").description("Description text").build();
/// let response_fields = Response::new().fields(vec![("Field title", "Field value", false)]).description("Description text").build();
/// ```
pub struct Response<'a> {
    title: Option<&'a str>,
    fields: Option<Vec<(&'a str, &'a str, bool)>>,
    description: Option<&'a str>,
}

impl<'a> Response<'a> {
    pub fn new() -> Self {
        Self {
            title: None,
            fields: None,
            description: None,
        }
    }
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
    pub fn fields(mut self, fields: Vec<(&'a str, &'a str, bool)>) -> Self {
        self.fields = Some(fields);
        self
    }
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
    pub fn build(self) -> CreateEmbed {
        let mut message_data: CreateEmbed = CreateEmbed::default();
        if let Some(title) = self.title {
            message_data = message_data.title(title);
        }
        if let Some(fields) = self.fields {
            message_data = message_data.fields(fields);
        }
        if let Some(description) = self.description {
            // Prevent sending messages with description over 4095 characters
            message_data = message_data.description(
                description
                    .get(0..=MAX_DESCRIPTION_SIZE)
                    .unwrap_or(description),
            );
        }
        message_data = message_data.colour(EMBED_COLOR);
        message_data
    }
}

impl<'a> Default for Response<'a> {
    fn default() -> Self {
        Self::new()
    }
}

// Common functions

/// Parses track position from string in format MM:SS
///
/// # Arguments
/// * `position` - position as MM:SS
///
/// # Examples
/// ```
/// use dj_ferris::parse_position;
/// use std::time::Duration;
/// let position = parse_position("01:02");
///
/// # assert_eq!(parse_position("01:02"), Duration::from_secs(1*60+2));
/// # assert_eq!(parse_position("01"), Duration::from_secs(1*60));
/// # assert_eq!(parse_position("1:2"), Duration::from_secs(1*60+2));
/// # assert_eq!(parse_position(":1"), Duration::from_secs(1));
/// ```
pub fn parse_position(position: &str) -> Duration {
    let time_vec: Vec<&str> = position.split(':').collect();
    let minutes = time_vec[0].parse::<u64>().unwrap_or(0);
    let seconds = if time_vec.len() > 1 {
        time_vec[1].parse::<u64>().unwrap_or(0)
    } else {
        0
    };

    Duration::from_secs(minutes * 60 + seconds)
}

pub fn parse_offset_position(
    current_position: Duration,
    sign: char,
    offset_duration: Duration,
) -> Duration {
    match sign {
        '+' => current_position.saturating_add(offset_duration),
        '-' => current_position.saturating_sub(offset_duration),
        _ => current_position,
    }
}

pub async fn get_current_position(ctx: &Context, guild_id: GuildId) -> Option<Duration> {
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>()?;
    let current_position = lava_client
        .get_player_context(guild_id.get())?
        .get_player()
        .await
        .ok()?
        .track?
        .info
        .position;
    Some(Duration::from_millis(current_position))
}

/// Check progress of current song and return a string with progress bar
///
/// # Arguments
/// * `duration` - length of the song as seconds
/// * `position` - current position in the song as seconds
///
///  # Examples
/// ```
/// use dj_ferris::get_progress;
/// let progress = get_progress(150, 300);
///
/// assert_eq!(progress, "[▮▮▮▮▮▮▮▮▮▮●▯▯▯▯▯▯▯▯▯]\t02:30/05:00")
/// ```
pub fn get_progress(position: u64, duration: u64) -> String {
    let bar_duration = duration / 20;
    let bar_position = position.checked_div(bar_duration).ok_or(0).unwrap_or(0);
    let mut bar: String = String::new();
    bar.push('[');
    for i in 0..20 {
        match i.cmp(&bar_position) {
            Ordering::Less => bar.push('▮'),
            Ordering::Greater => bar.push('▯'),
            Ordering::Equal => bar.push('●'),
        }
    }
    bar.push_str("]\t");
    let duration_min = format!("{:02}", duration / 60);
    let duration_sec = format!("{:02}", duration % 60);
    let spot_min = format!("{:02}", position / 60);
    let spot_sec = format!("{:02}", position % 60);
    write!(bar, "{spot_min}:{spot_sec}/{duration_min}:{duration_sec}")
        .unwrap_or_else(|_| warn!("Could not write to process bar"));
    bar
}

pub async fn get_queue(
    lava_client: &LavalinkClient,
    guild_id: GuildId,
) -> Result<QueueRef, FerrisError> {
    let queue = lava_client
        .get_player_context(guild_id.get())
        .ok_or(FerrisError::LavalinkError)?
        .get_queue();
    Ok(queue)
}
