use crate::{FerrisError, Lavalink};
use lavalink_rs::prelude::{LavalinkClient, PlayerContext};
use serenity::{
    all::{CommandDataOption, CommandDataOptionValue, CommandInteraction},
    client::Context,
    model::id::GuildId,
};
use songbird::Songbird;
use std::sync::Arc;

// Modules
pub mod goto;
pub mod help;
pub mod join;
pub mod leave;
pub mod loop_track;
pub mod np;
pub mod play;
pub mod queue;
pub mod skip;
pub mod stop;

pub async fn get_songbird_manager(ctx: &Context) -> Arc<Songbird> {
    songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
}

fn get_args(args: Vec<CommandDataOption>) -> Result<String, FerrisError> {
    if let CommandDataOptionValue::String(arg) = args[0].clone().value {
        Ok(arg)
    } else {
        Err(FerrisError::MissingArguments)
    }
}

pub async fn get_command_context<'a>(
    ctx: &'a Context,
    interaction: &'a CommandInteraction,
) -> Result<
    (
        GuildId,
        LavalinkClient,
        Option<PlayerContext>,
        Arc<Songbird>,
    ),
    FerrisError,
> {
    let guild_id = interaction.guild_id.ok_or(FerrisError::GuildError)?;
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().ok_or(FerrisError::LavalinkError)?;
    let player = lava_client.get_player_context(guild_id.get());
    Ok((
        guild_id,
        lava_client.clone(),
        player,
        get_songbird_manager(ctx).await,
    ))
}
