use super::get_command_context;
use crate::{FerrisError, FerrisResponse, LoopingTrack, MAX_DESCRIPTION_SIZE, Response, get_queue};
use lavalink_rs::player_context::TrackInQueue;
use serenity::{all::CommandInteraction, client::Context, futures::StreamExt};
use std::fmt::Write;
use tokio::sync::Mutex;

pub async fn queue(ctx: &Context, interaction: &CommandInteraction) -> FerrisResponse {
    // Init variables
    let (guild_id, lava_client, _, _) = get_command_context(ctx, interaction).await?;
    let queue = get_queue(&lava_client, guild_id)
        .await?
        .collect::<Vec<TrackInQueue>>()
        .await;

    if queue.is_empty() {
        Err(FerrisError::QueueEmptyError)?;
    }

    let mutex = lava_client.data::<Mutex<Option<LoopingTrack>>>()?;
    let looping = mutex.lock().await;

    // Construct queue
    let mut queue_string = String::new();
    for (index, track) in queue.into_iter().enumerate() {
        let (title, uri) = (
            track.track.info.title,
            track.track.info.uri.unwrap_or("Unknown".to_string()),
        );
        let mut track_string = format!("{index}. [{title}]({uri})");
        if looping.is_some() && index == 0 {
            track_string.push_str(" 🔁");
        }

        // Prevent writing to more tracks to queue string if we are nearing the message limit
        if (queue_string.len() + track_string.len()) > MAX_DESCRIPTION_SIZE {
            writeln!(queue_string, ". . .")?;
            break;
        }
        writeln!(queue_string, "{track_string}")?;
    }

    // Respond in Discord
    Ok(Response::new()
        .title("Queue")
        .description(&queue_string)
        .build())
}
