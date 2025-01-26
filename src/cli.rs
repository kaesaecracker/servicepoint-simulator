use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(
        long,
        default_value = "0.0.0.0:2342",
        help = "address and port to bind to"
    )]
    pub bind: String,
    #[arg(
        short,
        long,
        help = "Set default log level lower. You can also change this via the RUST_LOG environment variable."
    )]
    pub debug: bool,
    #[arg(
        short,
        long,
        help = "The name of the font family to use. This defaults to the system monospace font."
    )]
    pub font: Option<String>,
    #[clap(flatten)]
    pub gui: GuiOptions,
}

#[derive(Parser, Debug)]
pub struct GuiOptions {
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Use the red color channel"
    )]
    pub red: bool,
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Use the green color channel"
    )]
    pub green: bool,
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Use the blue color channel"
    )]
    pub blue: bool,
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "add spacers between tile rows to simulate gaps in real display"
    )]
    pub spacers: bool,
}
