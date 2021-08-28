use super::get_command_context;
use crate::{FerrisError, FerrisResponse, Response, get_progress};
use serenity::{all::CommandInteraction, client::Context};

pub async fn now_playing(ctx: &Context, interaction: &CommandInteraction) -> FerrisResponse {
    // Init variables
    let (_, _, Some(player), _) = get_command_context(ctx, interaction).await? else {
        Err(FerrisError::LavalinkError)?
    };
    let player = player.get_player().await?;
    let current_song = player.track.ok_or(FerrisError::QueueEmptyError)?;

    // Respond in Discord
    Ok(Response::new()
        .title("Now playing")
        .description(&format!(
            "{}\n {}",
            current_song.info.title,
            get_progress(
                player.state.position / 1000,
                current_song.info.length / 1000,
            )
        ))
        .build())
}
