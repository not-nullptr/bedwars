use tokio::io::AsyncWriteExt;

use crate::{Gamemode, Writable};

#[derive(Debug, Clone, Copy)]
pub enum GameEvent {
    NoRespawnBlockAvailable,
    BeginRaining,
    EndRaining,
    ChangeGamemode(Gamemode),
    WinGame(ShouldRollCredits),
    DemoEvent(DemoEvent),
    ArrowHitPlayer,
    RainLevelChange(f32),
    ThunderLevelChange(f32),
    PlayPufferfishSting,
    ElderGuardianMobAppearance,
    EnableRespawnScreen(RespawnScreenKind),
    LimitedCrafting(LimitedCraftingKind),
    StartWaitingForLevelChunks,
}

#[derive(Debug, Clone, Copy)]
pub enum ShouldRollCredits {
    DontRollCredits = 0,
    RollCredits = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum DemoEvent {
    ShowWelcomeScreen = 0,
    TellMovementControls = 101,
    TellJumpControl = 102,
    TellInventoryControl = 103,
    TellDemoIsOver = 104,
}

#[derive(Debug, Clone, Copy)]
pub enum RespawnScreenKind {
    Enabled = 0,
    ImmediateRespawn = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum LimitedCraftingKind {
    Disabled = 0,
    Enabled = 1,
}

impl Writable for GameEvent {
    async fn write_into<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        let (discrim, val) = match self {
            GameEvent::NoRespawnBlockAvailable => (0, 0.0),
            GameEvent::BeginRaining => (1, 0.0),
            GameEvent::EndRaining => (2, 0.0),
            GameEvent::ChangeGamemode(gamemode) => (3, *gamemode as u8 as f32),
            GameEvent::WinGame(should_roll_credits) => (4, *should_roll_credits as u8 as f32),
            GameEvent::DemoEvent(demo_event) => (5, *demo_event as u8 as f32),
            GameEvent::ArrowHitPlayer => (6, 0.0),
            GameEvent::RainLevelChange(level) => (7, *level),
            GameEvent::ThunderLevelChange(level) => (8, *level),
            GameEvent::PlayPufferfishSting => (9, 0.0),
            GameEvent::ElderGuardianMobAppearance => (10, 0.0),
            GameEvent::EnableRespawnScreen(kind) => (11, *kind as u8 as f32),
            GameEvent::LimitedCrafting(kind) => (12, *kind as u8 as f32),
            GameEvent::StartWaitingForLevelChunks => (13, 0.0),
        };

        writer.write_u8(discrim).await?;
        writer.write_f32(val).await?;

        Ok(())
    }
}
