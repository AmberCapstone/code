use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::FutureExt;
use proto::sensor::{Command, Status};
use ratatui::{
    Terminal,
    prelude::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
};
use tokio::{
    select,
    sync::mpsc::{self, Receiver},
    task::JoinSet,
};
use tokio_serial::UsbPortInfo;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (status_tx, status_rx) = mpsc::channel::<Status>(100);
    let (_command_tx, command_rx) = mpsc::channel::<Command>(100); // Unused

    let stop = CancellationToken::new();

    let mut services = JoinSet::<anyhow::Result<()>>::new();
    services.spawn(serial::run(command_rx, status_tx, is_base_station, stop.clone()).map(Ok));
    services.spawn(printer(status_rx, stop.clone()));
    services.spawn(input_handler(stop.clone()));

    while services.join_next().await.is_some() {
        stop.cancel();
    }

    services.join_all().await;

    Ok(())
}

async fn input_handler(stop: CancellationToken) -> anyhow::Result<()> {
    stop.run_until_cancelled(async {
        loop {
            let event = tokio::task::spawn_blocking(event::read).await??;
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) = event
            {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => break,
                    _ => {}
                }
            }
        }

        Ok(())
    })
    .await
    .unwrap_or(Ok(()))
}

async fn printer(mut status_rx: Receiver<Status>, stop: CancellationToken) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        select! {
            status = status_rx.recv() => {
                let text = format!("{status:#?}");
                terminal.draw(|f| {
                    let area = f.area();
                    let para = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
                    f.render_widget(para, area);
                })?;
            }

            () = stop.cancelled() => {
                break;
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

fn is_base_station(info: &UsbPortInfo) -> bool {
    info.manufacturer.as_deref() == Some("Amber") && info.product.as_deref() == Some("Base Station")
}
