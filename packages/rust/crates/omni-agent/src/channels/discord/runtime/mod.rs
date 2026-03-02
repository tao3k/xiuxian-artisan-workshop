//! Discord runtime wiring (ingress + foreground turn execution).

mod config;
mod dispatch;
mod foreground;
mod gateway;
mod ingress;
mod interrupt;
mod managed;
mod run;
mod telemetry;

pub use config::DiscordRuntimeConfig;
pub use gateway::run_discord_gateway;
pub use ingress::{
    DiscordIngressApp, DiscordIngressBuildRequest, build_discord_ingress_app,
    build_discord_ingress_app_with_control_command_policy,
    build_discord_ingress_app_with_partition_and_control_command_policy,
};
pub use run::{DiscordIngressRunRequest, run_discord_ingress};

pub(in crate::channels::discord::runtime) use interrupt::ForegroundInterruptController;
