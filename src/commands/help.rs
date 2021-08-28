use crate::{FerrisResponse, Response};
use serenity::{all::CommandInteraction, prelude::Context};

const HELP_FIELDS: [(&str, &str, bool); 12] = [
    ("/join", "Joins your current channel", false),
    ("/leave", "Leaves current channel", false),
    (
        "/play {url / query}",
        "Plays audio from url or plays the first search result",
        false,
    ),
    ("/stop", "Stops current song and clears queue", false),
    ("/queue", "Shows audio in queue", false),
    ("/skip", "Skips to next track", false),
    ("/help", "Shows this message", false),
    (
        "/loop",
        "Enable or disable looping for current track",
        false,
    ),
    ("/np", "Shows current song and its progress", false),
    ("/goto {M:S}", "Goes to specified point in the track", false),
    (
        "Issues?",
        "If the bot has an issue or doesn't work you can try **stop** or **leave** commands to reset the bot. You can also check known issues at [DJ Ferris repository](https://github.com/KvanttoriOy/dj_ferris/issues)",
        false,
    ),
    ("Version:", env!("CARGO_PKG_VERSION"), false),
];

pub async fn help(_ctx: &Context, _interaction: &CommandInteraction) -> FerrisResponse {
    // Respond in Discord
    Ok(Response::new()
        .title("Help")
        .fields(HELP_FIELDS.into())
        .description("Below is a list of commands")
        .build())
}
