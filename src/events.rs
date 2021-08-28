use crate::{
    FerrisError, Lavalink, LoopingTrack, Response,
    commands::{self, get_songbird_manager},
};
use lavalink_rs::model::events::TrackStart;
use lavalink_rs::prelude::LavalinkClient;
use serenity::{
    all::{CommandOptionType, Interaction},
    async_trait,
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    client::{Context, EventHandler},
    framework::standard::macros::hook,
    gateway::ActivityData,
    model::gateway::Ready,
    model::prelude::VoiceState,
};
use tokio::sync::Mutex;
use tracing::{info, log::error, warn};

pub struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let message_data_result = match command.data.name.as_str() {
                "help" => commands::help::help(&ctx, &command).await,
                "join" => commands::join::join(&ctx, &command).await,
                "play" => commands::play::play(&ctx, &command).await,
                "leave" => commands::leave::leave(&ctx, &command).await,
                "stop" => commands::stop::stop(&ctx, &command).await,
                "loop" => commands::loop_track::loop_track(&ctx, &command).await,
                "queue" => commands::queue::queue(&ctx, &command).await,
                "skip" => commands::skip::skip(&ctx, &command).await,
                "np" => commands::np::now_playing(&ctx, &command).await,
                "goto" => commands::goto::goto(&ctx, &command).await,
                _ => Err("No such command".into()),
            };

            if let Err(why) = command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message({
                        CreateInteractionResponseMessage::new().add_embed(
                            message_data_result.unwrap_or_else(|err| {
                                let desciption =
                                    if let Some(error) = err.downcast_ref::<FerrisError>() {
                                        warn!("{error:#?}");
                                        error.to_string()
                                    } else {
                                        error!("{err:#?}");
                                        "Something went wrong".to_string()
                                    };
                                Response::new()
                                    .title("Error")
                                    .description(&desciption)
                                    .build()
                            }),
                        )
                    }),
                )
                .await
            {
                error!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Some(ActivityData::listening("/help")));

        info!("{} is connected!", ready.user.name);

        serenity::model::application::Command::set_global_commands(
            ctx.http,
            vec![
                CreateCommand::new("join").description("Joins current channel"),
                CreateCommand::new("leave").description("Leaves current channel"),
                CreateCommand::new("play")
                    .description("Queues a track")
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "url",
                            "url of the track or a search query",
                        )
                        .required(true),
                    ),
                CreateCommand::new("goto")
                    .description("Goes to specific point in track")
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "position",
                            "position in M:S fformat",
                        )
                        .required(true),
                    ),
                CreateCommand::new("stop").description("Stops current track and clears queue"),
                CreateCommand::new("queue").description("Displays current queue"),
                CreateCommand::new("skip").description("Skips current song"),
                CreateCommand::new("help").description("Displays help"),
                CreateCommand::new("loop").description("Loops current track"),
                CreateCommand::new("np").description("Displays info on currently playing track"),
            ],
        )
        .await
        .expect("Expected to be able to create commands");
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        // Only disconnect and destroy player if we have an old voice state i.e Ferris is on a voice channel and if Ferris is the member disconnecting
        if new.member.is_some_and(|member| {
            member.user.id.to_string()
                == dotenvy::var("APPLICATION_ID").expect("Missing application ID")
        }) && let Some(old_state) = old
            && let Some(guild_id) = old_state.guild_id
        {
            // Check if we are already on a voice channel
            let manager = get_songbird_manager(&ctx).await;
            if manager.get(guild_id).is_some()
                && let Err(e) = manager.remove(guild_id).await
            {
                error!("Removing manager failed with error: {}", e)
            }

            ctx.data
                .read()
                .await
                .get::<Lavalink>()
                .expect("Expected to have lavalink client in voice state update")
                .delete_player(guild_id.get())
                .await
                .expect("Failed to delete player")
        }
    }
}

#[hook]
pub async fn track_start(client: LavalinkClient, _session_id: String, event: &TrackStart) {
    let mutex = client.data::<Mutex<Option<LoopingTrack>>>().unwrap();
    let data = mutex.lock().await;
    let player_context = client.get_player_context(event.guild_id);

    if let (Some(player), Some(looping_track)) = (player_context, &*data) {
        player
            .get_queue()
            .push_to_front(looping_track.0.clone())
            .unwrap_or(());
    }
}
