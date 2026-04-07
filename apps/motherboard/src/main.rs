use std::time::Duration;

use anyhow::anyhow;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
        MouseEventKind,
    },
    execute,
};
use futures::{FutureExt, TryFutureExt};
use proto::base_station::{Command, Status};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use tokio::{
    select,
    sync::mpsc::{self, Receiver},
    task::JoinSet,
};
use tokio_serial::UsbPortInfo;
use tokio_util::sync::CancellationToken;

mod backscatter_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (status_tx, status_rx) = mpsc::channel::<Status>(100);
    let (backscatter_tx, backscatter_rx) = mpsc::channel::<Status>(100);
    let (printer_tx, printer_rx) = mpsc::channel::<Status>(100);
    let (_command_tx, command_rx) = mpsc::channel::<Command>(100); // Unused

    let stop = CancellationToken::new();

    let mut services = JoinSet::<anyhow::Result<()>>::new();
    services.spawn(fan(status_rx, vec![backscatter_tx, printer_tx], stop.clone()));
    services.spawn(serial::run(command_rx, status_tx, is_base_station, stop.clone()).map(Ok));
    services.spawn(backscatter_server::run(backscatter_rx, stop.clone()).map_err(anyhow::Error::from));
    services.spawn(tui(printer_rx, stop.clone()));
    services.spawn(input_handler(stop.clone()));

    while services.join_next().await.is_some() {
        stop.cancel();
    }

    services.join_all().await;

    Ok(())
}

async fn fan<T: Clone>(
    mut input: mpsc::Receiver<T>,
    outputs: Vec<mpsc::Sender<T>>,
    stop: CancellationToken,
) -> anyhow::Result<()> {
    stop.run_until_cancelled(async {
        loop {
            let msg = input.recv().await.ok_or(anyhow!("Input closed"))?;

            for out in &outputs {
                let _ = out.try_send(msg.clone());
            }
        }
    })
    .await
    .unwrap_or(Ok(()))
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

async fn tui(mut status_rx: Receiver<Status>, stop: CancellationToken) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    execute!(terminal.backend_mut(), EnableMouseCapture)?;

    let mut scroll: u16 = 0;

    let mut last_status: Option<Status> = None;

    'tui: loop {
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break 'tui,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break 'tui,
                    _ => (),
                },
                Event::Mouse(m) => match m.kind {
                    MouseEventKind::ScrollDown => scroll = scroll.saturating_add(3),
                    MouseEventKind::ScrollUp => scroll = scroll.saturating_sub(3),
                    _ => (),
                },
                _ => (),
            }
        }

        select! {
            Some(s) = status_rx.recv() => {
                last_status = Some(s);
            },
            () = stop.cancelled() => break,
            () = tokio::time::sleep(Duration::from_millis(50)) => {}
        }

        terminal.draw(|f| {
            let alt_style = Style::default().fg(Color::LightGreen);

            let [title_area, body_area, footer_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1)]).areas(f.area());

            let title = Paragraph::new(if last_status.is_some() {
                "Connected to Base Station"
            } else {
                "Not connected to Base Station"
            })
            .style(alt_style);

            let para = Paragraph::new(last_status.map(|s| format!("{s:#?}")).unwrap_or_default())
                .scroll((scroll, 0))
                .block(Block::default().borders(Borders::all()));

            let footer = Paragraph::new("q: quit").style(alt_style);

            f.render_widget(title, title_area);
            f.render_widget(para, body_area);
            f.render_widget(footer, footer_area);
        })?;
    }

    stop.cancel();
    let r = execute!(terminal.backend_mut(), DisableMouseCapture);
    ratatui::restore();

    r.map_err(anyhow::Error::from)
}

fn is_base_station(info: &UsbPortInfo) -> bool {
    info.manufacturer.as_deref() == Some("Amber") && info.product.as_deref() == Some("Base Station")
}
