use notan::{draw::DrawConfig, log, prelude::*};
use superuser::{draw, event, setup, update};

#[notan_main]
fn main() -> Result<(), String> {
    notan::init_with(setup)
        .add_config(DrawConfig)
        .add_config(log::LogConfig::debug())
        .event(event)
        .update(update)
        .draw(draw)
        .build()
}
