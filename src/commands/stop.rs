use super::get_command_context;
use crate::{FerrisError, FerrisResponse, Response};
use serenity::{all::CommandInteraction, client::Context};

pub async fn stop(ctx: &Context, interaction: &CommandInteraction) -> FerrisResponse {
    // Init variables
    let (_guild_id, _lava_client, Some(player), _) = get_command_context(ctx, interaction).await?
    else {
        Err(FerrisError::LavalinkError)?
    };

    // Send command to Lavalink
    player.stop_now().await?;
    player.get_queue().clear()?;

    Ok(Response::new().description("Queue cleared").build())
}
