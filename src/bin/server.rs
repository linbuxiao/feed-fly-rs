use teloxide::{prelude::*, utils::command::BotCommands};

use feed_fly_rs::usecase::{TgBot, UseCase};
use teloxide::types::ParseMode::Html;
use feed_fly_rs::config;

#[tokio::main]
async fn main() {
    // https://github.com/seanmonstar/pretty-env-logger
    pretty_env_logger::init();
    log::info!("Start Feed Fly Rust!");
    // https://github.com/teloxide/teloxide
    let cfg = config::default_config().await;
    let bot = Bot::new(cfg.bot_token).auto_send();

    let handler = Update::filter_message().branch(
        dptree::entry()
            .filter_command::<AllCommands>()
            .endpoint(commands_handler),
    );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
        })
        .build()
        .dispatch()
        .await;
}

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "Simple commands")]
enum AllCommands {
    #[command(description = "start all.")]
    Start,
    #[command(description = "list all your feeds.")]
    List,
}

async fn commands_handler(
    msg: Message,
    bot: AutoSend<Bot>,
    cmd: AllCommands,
) -> Result<(), teloxide::RequestError> {
    let cfg = config::default_config().await;
    let uc = UseCase::new();
    let tg = TgBot::new(&bot, uc, cfg.bot_token, cfg.telegram_id);
    let user_id = msg.chat.id;
    let text = match cmd {
        AllCommands::Start => tg.start(),
        AllCommands::List => tg.list().await,
    };
    bot.send_message(user_id, text)
        .disable_web_page_preview(true)
        .parse_mode(Html)
        .await?;

    Ok(())
}
