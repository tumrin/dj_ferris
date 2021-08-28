use super::get_command_context;
use crate::{FerrisError, FerrisResponse, Response};
use serenity::all::CommandInteraction;
use serenity::client::Context;

pub async fn leave(ctx: &Context, interaction: &CommandInteraction) -> FerrisResponse {
    // Init variables
    let (guild_id, lava_client, _, manager) = get_command_context(ctx, interaction).await?;

    // Ćheck if on call
    if manager.get(guild_id).is_none() {
        Err(FerrisError::NotOnCallError)?
    }

    // Remove from call
    manager.remove(guild_id).await?;

    // Send command to lavalink
    lava_client.delete_player(guild_id.get()).await?;

    // Respond in Discord
    Ok(Response::new().description("Left voice channel").build())
}
