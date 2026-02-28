use colored::Colorize;
use log::{Level, LevelFilter};
use std::time::{SystemTime, UNIX_EPOCH};

fn level_badge(level: Level) -> String {
    match level {
        Level::Error => " ERROR ".white().on_red().bold().to_string(),
        Level::Warn => " WARN  ".black().on_yellow().bold().to_string(),
        Level::Info => " INFO  ".black().on_green().bold().to_string(),
        Level::Debug => " DEBUG ".white().on_blue().bold().to_string(),
        Level::Trace => " TRACE ".white().on_magenta().bold().to_string(),
    }
}

pub fn init_logger() -> Result<(), log::SetLoggerError> {
    let configured_level = std::env::var("LOG_LEVEL")
        .ok()
        .and_then(|raw| raw.parse::<LevelFilter>().ok())
        .unwrap_or(LevelFilter::Info);

    fern::Dispatch::new()
        .level(configured_level)
        .level_for("actix_web", LevelFilter::Info)
        .format(|out, message, record| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            let timestamp = format!("{:>10}.{:03}", now.as_secs(), now.subsec_millis()).dimmed();
            let line = record
                .line()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".to_string());

            out.finish(format_args!(
                "{} {} {}:{} {}",
                timestamp,
                level_badge(record.level()),
                record.target().cyan(),
                line.cyan(),
                message
            ));
        })
        .chain(std::io::stdout())
        .apply()
}
