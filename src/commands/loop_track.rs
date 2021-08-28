use super::get_command_context;
use crate::{FerrisError, FerrisResponse, LoopingTrack, Response};
use serenity::{all::CommandInteraction, client::Context};
use tokio::sync::Mutex;

pub async fn loop_track(ctx: &Context, interaction: &CommandInteraction) -> FerrisResponse {
    let (_, lava_client, Some(player), _) = get_command_context(ctx, interaction).await? else {
        Err(FerrisError::LavalinkError)?
    };

    let mutex = lava_client.data::<Mutex<Option<LoopingTrack>>>()?;
    let mut data = mutex.lock().await;

    // Try to get current song and return error if this fails
    let current_song = player
        .get_player()
        .await?
        .track
        .ok_or(FerrisError::QueueEmptyError)?;

    // If we are looping, remove looping track, else add current track as looping track
    let description = if data.is_some() {
        (*data) = None;
        format!("Stop looping {}", current_song.info.title)
    } else {
        (*data) = Some(LoopingTrack(current_song.clone()));
        player
            .get_queue()
            .push_to_front(current_song.clone())
            .unwrap_or(());
        format!("Start looping {}", current_song.info.title)
    };

    // Respond in Discord
    Ok(Response::new().description(&description).build())
}
