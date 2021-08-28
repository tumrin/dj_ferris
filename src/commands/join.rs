use super::get_command_context;
use crate::{FerrisError, FerrisResponse, Response};
use lavalink_rs::model::player::ConnectionInfo;
use serenity::{
    all::{CommandInteraction, Mentionable},
    client::Context,
};

pub async fn join(ctx: &Context, interaction: &CommandInteraction) -> FerrisResponse {
    // Init variables
    let (guild_id, lava_client, _, manager) = get_command_context(ctx, interaction).await?;
    let guild = ctx.cache.guild(guild_id);
    let channel_id = guild
        .expect("Expected guild")
        .voice_states
        .get(&interaction.user.id)
        .and_then(|voice_state| voice_state.channel_id)
        .ok_or(FerrisError::NotOnCallError)?;

    //Check if already on a channel
    if manager.get(guild_id).is_some() {
        Err(FerrisError::AlreadyOnCallError)?
    }

    // Send command to Lavalink
    let connection_info = manager.join_gateway(guild_id, channel_id).await?.0; // Call is discarded since we don't need it
    lava_client
        .create_player_context(
            guild_id.get(),
            ConnectionInfo {
                endpoint: connection_info.endpoint,
                token: connection_info.token,
                session_id: connection_info.session_id,
            },
        )
        .await?;

    // Respond in Discord
    Ok(Response::new()
        .description(&format!("Joined {}", channel_id.mention()))
        .build())
}
