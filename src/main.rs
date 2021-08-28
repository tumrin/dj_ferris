use dj_ferris::{
    Lavalink, LoopingTrack,
    events::{Handler, track_start},
};
use lavalink_rs::{
    client::LavalinkClient,
    model::{client::NodeDistributionStrategy, events},
    prelude::NodeBuilder,
};
#[allow(deprecated)]
use serenity::framework::StandardFramework;
use serenity::{client::Client, prelude::GatewayIntents};
use songbird::{Config, SerenityInit};
use std::{env, sync::Arc};
use tokio::sync::Mutex;
use tracing::{Level, log::error};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Set default log level to INFO if not specified with RUST_LOG env variable
    tracing_subscriber::fmt()
        .with_env_filter(match EnvFilter::try_from_default_env() {
            Ok(filter) => filter,
            Err(_err) => EnvFilter::default().add_directive(Level::INFO.into()),
        })
        .init();

    // Configure the client with your Discord bot token in the environment.
    let token: String = dotenvy::var("DISCORD_TOKEN").unwrap_or_else(|_| {
        env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in .env or environment variable")
    });
    let lavalink_server_pass: String =
        dotenvy::var("LAVALINK_SERVER_PASSWORD").unwrap_or_else(|_| {
            env::var("LAVALINK_SERVER_PASSWORD")
                .expect("Expected LAVALINK_SERVER_PASSWORD in .env or environment variable")
        });
    #[allow(deprecated)]
    let framework = StandardFramework::new();
    let config = Config::default();
    let application_id = dotenvy::var("APPLICATION_ID")
        .unwrap_or_else(|_| {
            env::var("APPLICATION_ID")
                .expect("Expected APPLICATION_ID in .env or environment variable")
        })
        .parse()
        .expect("Could not parse APPLICATION_ID, check that it's a number");
    let mut client = Client::builder(&token, GatewayIntents::non_privileged())
        .event_handler(Handler)
        .application_id(application_id)
        .framework(framework)
        .register_songbird_from_config(config)
        .await
        .expect("Error creating serenity client");

    let nodes = NodeBuilder {
        hostname: "localhost:2333".to_string(),
        is_ssl: false,
        events: events::Events::default(),
        user_id: lavalink_rs::model::UserId(
            client
                .http
                .get_current_user()
                .await
                .expect("Could not get current user")
                .id
                .into(),
        ),
        password: lavalink_server_pass,
        session_id: None,
    };
    let events = events::Events {
        track_start: Some(track_start),
        ..Default::default()
    };

    let looping: Option<LoopingTrack> = None;
    let lava_client = LavalinkClient::new_with_data(
        events,
        vec![nodes],
        NodeDistributionStrategy::default(),
        Arc::new(Mutex::new(looping)),
    )
    .await;

    // This block is here to release RWLock after lavalink client has been added to data
    {
        let mut data = client.data.write().await;
        data.insert::<Lavalink>(lava_client.clone());
    }

    client
        .start()
        .await
        .map_err(|why| error!("Client ended: {:?}", why))
        .expect("Failed to start");
}
