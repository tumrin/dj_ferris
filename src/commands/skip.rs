use super::get_command_context;
use crate::{FerrisError, FerrisResponse, LoopingTrack, Response, get_queue};
use serenity::{all::CommandInteraction, client::Context};
use tokio::sync::Mutex;

pub async fn skip(ctx: &Context, interaction: &CommandInteraction) -> FerrisResponse {
    // Init variables
    let (guild_id, lava_client, Some(player), _) = get_command_context(ctx, interaction).await?
    else {
        Err(FerrisError::LavalinkError)?
    };

    let mutex = lava_client.data::<Mutex<Option<LoopingTrack>>>()?;
    let mut data = mutex.lock().await;
    let final_track = get_queue(&lava_client, guild_id).await?.get_count().await? == 0;

    let track = player
        .get_player()
        .await?
        .track
        .ok_or(FerrisError::QueueEmptyError)?;

    // If we are looping, remove looping track
    if data.is_some() {
        (*data) = None;
    }

    // Send command to Lavalink
    player.skip()?;

    // If final stop playing and restart player
    if final_track {
        player.stop_now().await?; // Stops client
    }

    // Respond in Discord
    Ok(Response::new()
        .description(&format!("Skipped {}", track.info.title))
        .build())
}
