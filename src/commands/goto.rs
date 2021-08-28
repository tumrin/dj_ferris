use super::{get_args, get_command_context};
use crate::{
    FerrisError, FerrisResponse, Response, get_current_position, parse_offset_position,
    parse_position,
};
use serenity::{all::CommandInteraction, client::Context, model::id::GuildId};
use std::time::Duration;

pub async fn goto(ctx: &Context, interaction: &CommandInteraction) -> FerrisResponse {
    // Init variables
    let (guild_id, _, Some(player), _) = get_command_context(ctx, interaction).await? else {
        Err(FerrisError::LavalinkError)?
    };
    let position =
        get_goto_position(&get_args(interaction.data.options.clone())?, ctx, guild_id).await?;

    // Check that something is playing
    if player
        .get_player()
        .await
        .is_ok_and(|player| player.track.is_none())
    {
        //Return early if queue is empty
        return Err(FerrisError::QueueEmptyError)?;
    }

    // Send command to Lavalink
    player.set_position(position).await?;

    // Respond in Discord
    Ok(Response::new()
        .description(&format!(
            "Go to {:0>2}:{:0>2}",
            position.as_secs() / 60,
            position.as_secs() % 60
        ))
        .build())
}

/// Get goto jump position from arguments.
///
/// # Arguments
///
/// * `position_arg` - parsed argument given to the bot with goto command
/// * `ctx` - Serenity context
/// * `guild_id` - ID of the Discord server
///
pub async fn get_goto_position(
    position_arg: &str,
    ctx: &Context,
    guild_id: GuildId,
) -> Result<Duration, FerrisError> {
    // Run either absolute or relative goto based on arguments
    if position_arg.starts_with('+') || position_arg.starts_with('-') {
        //Relative goto
        let current_position = get_current_position(ctx, guild_id)
            .await
            .ok_or(FerrisError::PositionError)?;
        Ok(parse_offset_position(
            current_position,
            position_arg
                .chars()
                .next()
                .ok_or(FerrisError::PositionError)?,
            parse_position(position_arg.split_at(1).1),
        ))
    } else {
        // Absolute goto
        Ok(parse_position(position_arg))
    }
}
