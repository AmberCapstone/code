use std::time::Duration;

use amber_connect::{self, codec::PbReceiver};
use crossterm::{
    event::{self, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind},
    execute,
};
use proto::sensor::Status;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use tokio::{select, sync::watch, task::JoinSet, time::timeout};
use tokio_util::sync::CancellationToken;
use zeromq::{Socket, SubSocket};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stop = CancellationToken::new();

    let (status_tx, status_rx) = watch::channel::<Option<Status>>(None);

    let mut services = JoinSet::new();
    services.spawn(reader(status_tx, stop.clone()));
    services.spawn(tui(status_rx, stop.clone()));

    while services.join_next().await.is_some() {
        stop.cancel();
    }

    Ok(())
}

async fn tui(mut status_rx: watch::Receiver<Option<Status>>, stop: CancellationToken) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    execute!(terminal.backend_mut(), EnableMouseCapture)?;

    let mut scroll: u16 = 0;

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
            Ok(()) = status_rx.changed() => {}
            () = stop.cancelled() => break,
            () = tokio::time::sleep(Duration::from_millis(50)) => {}
        }

        terminal.draw(|f| {
            let alt_style = Style::default().fg(Color::LightGreen);

            let status = status_rx.borrow().clone();

            let [title_area, body_area, footer_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1)]).areas(f.area());

            let title = Paragraph::new({
                if let Some(status) = &status {
                    format!("Connected to {}", status.name.clone().unwrap_or("-".to_string()))
                } else {
                    "No Sensor Boards connected".to_string()
                }
            })
            .style(alt_style);

            let para = Paragraph::new(status.map(|s| format!("{s:#?}")).unwrap_or_default())
                .scroll((scroll, 0))
                .block(Block::default().borders(Borders::all()));

            let footer = Paragraph::new("q: quit").style(alt_style);

            f.render_widget(title, title_area);
            f.render_widget(para, body_area);
            f.render_widget(footer, footer_area);
        })?;
    }

    stop.cancel();
    ratatui::restore();
    Ok(())
}

async fn reader(tx: watch::Sender<Option<Status>>, stop: CancellationToken) -> anyhow::Result<()> {
    let mut status_sock = SubSocket::new();
    status_sock.connect(amber_connect::endpoint::STATUS).await?;
    status_sock.subscribe("").await?;

    stop.run_until_cancelled(async {
        loop {
            let to_send = if let Ok(r) = timeout(Duration::from_secs(1), status_sock.recv_msg::<Status>()).await {
                if let Ok(new_status) = r {
                    Some(new_status)
                } else {
                    break;
                }
            } else {
                None
            };

            if tx.send(to_send).is_err() {
                break;
            }
        }
    })
    .await;

    status_sock.close().await;

    Ok(())
}
