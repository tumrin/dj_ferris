use super::{get_args, get_command_context, join::join};
use crate::{FerrisError, FerrisResponse, Response};
use lavalink_rs::{
    model::track::TrackLoadType,
    prelude::{SearchEngines, TrackInQueue, TrackLoadData},
};
use serenity::{all::CommandInteraction, client::Context};

pub async fn play(ctx: &Context, interaction: &CommandInteraction) -> FerrisResponse {
    // Return error if Join gives some other error then already being on call
    if let Err(error) = join(ctx, interaction).await {
        let ferris_error = *error.downcast::<FerrisError>()?;
        if !matches!(ferris_error, FerrisError::AlreadyOnCallError) {
            return Err(ferris_error)?;
        }
    }

    // Init variables
    let (guild_id, lava_client, Some(player), _) = get_command_context(ctx, interaction).await?
    else {
        Err(FerrisError::LavalinkError)?
    };
    let url = get_args(interaction.data.options.clone())?;
    let query = if url.starts_with("http") {
        url.to_string()
    } else {
        SearchEngines::YouTube.to_query(&url)?
    };
    let query_result = lava_client.load_tracks(guild_id.get(), &query).await?;

    // Queue tracks
    let mut playlist_info = None;
    let tracks: Vec<TrackInQueue> = match query_result.data {
        Some(TrackLoadData::Track(track)) => vec![track.into()],
        Some(TrackLoadData::Search(results)) => vec![results[0].clone().into()],
        Some(TrackLoadData::Playlist(playlist)) => {
            playlist_info = Some(playlist.info);
            playlist
                .tracks
                .into_iter()
                .map(|track| track.into())
                .collect()
        }
        _ => Err(FerrisError::TrackNotFoundError)?,
    };

    // Send command to Lavalink
    player.get_queue().append(tracks.clone().into())?;

    // Get name and url of the track
    let (name, url) = match query_result.load_type {
        TrackLoadType::Playlist => (
            playlist_info.map_or("Unknown".to_string(), |info| info.name),
            None,
        ),
        TrackLoadType::Track | TrackLoadType::Search => {
            let track = tracks[0].clone().track;
            (track.info.title, track.info.uri)
        }
        _ => ("Unknown".to_string(), None),
    };

    // Respond in Discord
    let desciption = if let Some(url) = url {
        format!("Added [{name}]({url}) to queue")
    } else {
        format!("Added {name} to queue")
    };
    Ok(Response::new().description(&desciption).build())
}
