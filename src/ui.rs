use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};

use crate::app::App;
use crate::disassemble::disassemble;

pub fn ui(f: &mut Frame, app: &App) {
    // Begin by splitting the terminals into the chunks that we will use to display various parts
    // of the ui.
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(2), Constraint::Length(4)])
        .split(f.size());
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25),
            Constraint::Min(16),
            Constraint::Length(15),
        ])
        .split(vertical_chunks[0]);

    // The command chunk will display the command prompt.
    let command_prompt_chunk = vertical_chunks[1];
    // The registers chunk will display the contents of the CPU's registers.
    let registers_chunk = horizontal_chunks[2];
    // The RAM chunk will display the contents of RAM in the vicinity of the program counter.
    let ram_chunk = horizontal_chunks[1];
    // The instructions chunk will display a list of recently executed instructions, as well as the
    // instruction which will be executed on the next step.
    let instruction_history_chunk = horizontal_chunks[0];

    // Call all of the rendering functions.
    render_command_prompt(f, app, command_prompt_chunk);
    render_registers(f, app, registers_chunk);
    render_ram(f, app, ram_chunk);
    render_instruction_history(f, app, instruction_history_chunk);
}

// Render the current status of the registers in a given area of the frame.
pub fn render_registers(f: &mut Frame, app: &App, rect: Rect) {
    // The block in which the registers are displayed.
    let block = Block::default()
        .title("Registers")
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1));

    // These lines contain the actual data within the general purpose registers.
    let mut lines: Vec<Line> = app
        .cpu
        .registers
        .iter()
        .zip(0..32)
        .map(|(c, a)| {
            Line::from(vec![
                Span::styled(format!("r{:02}: ", a), Style::default()),
                if a == 0 {
                    Span::styled("0x0000", Style::default())
                } else {
                    Span::styled(format!("{:#06x}", c), Style::default())
                },
            ])
        })
        .collect();

    // Add a line of blank space between the general purpose registers and the program counter.
    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("pc:  ", Style::default()),
        Span::styled(
            format!("{:#06x}", app.cpu.program_counter),
            Style::default(),
        ),
    ]));

    let paragraph = Paragraph::new(lines).block(block).centered();
    f.render_widget(paragraph, rect);
}

// Render a snapshot of RAM containing the program counter in a given area of the frame.
pub fn render_ram(f: &mut Frame, app: &App, rect: Rect) {
    // The block in which the snapshot of RAM is displayed.
    let block = Block::default()
        .title("RAM")
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1));

    // There's a bit of annoying math to be done to determine which page of RAM ought to be
    // displayed.
    let inner = block.inner(rect);
    let page_size = inner.height * ((inner.width - 7) / 7);

    let mut lines = Vec::new();
    for row in 0..inner.height {
        let base = (app.cpu.program_counter / page_size) * page_size;
        let mut spans = vec![Span::styled(
            format!("{:#06x}:", base + row * ((inner.width - 7) / 7)),
            Style::default(),
        )];
        for column in 0..((inner.width - 7) / 7) {
            let address = base + row * ((inner.width - 7) / 7) + column;
            if address == app.cpu.program_counter {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("{:#06x}", app.cpu.ram[usize::from(address)]),
                    Style::default().fg(Color::Black).bg(Color::White),
                ));
            } else {
                spans.push(Span::styled(
                    format!(" {:#06x}", app.cpu.ram[usize::from(address)]),
                    Style::default(),
                ));
            }
        }
        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(paragraph, rect);
}

// Render the current command prompt.
pub fn render_command_prompt(f: &mut Frame, app: &App, rect: Rect) {
    // The block in which the command prompt is displayed.
    let block = Block::default()
        .title("Command Prompt")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(vec![
        match &app.command_result {
            Ok(message) => Line::styled(format!("{}", message), Style::default().fg(Color::Green)),
            Err(error) => Line::styled(format!("{}", error), Style::default().fg(Color::Red)),
        },
        Line::from(vec![
            Span::raw(format!("> {}", app.command_buffer)),
            Span::styled("█", Style::default().add_modifier(Modifier::SLOW_BLINK)),
        ]),
    ])
    .block(block);

    f.render_widget(paragraph, rect);
}

// Render the instruction history.
pub fn render_instruction_history(f: &mut Frame, app: &App, rect: Rect) {
    // The block in which the command prompt is displayed.
    let block = Block::default()
        .title("Instruction History")
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1));

    // We have to do a bit of math to figure out how much of the history to display.
    let inner = block.inner(rect);

    let mut lines = vec![Line::styled(
        disassemble(
            app.cpu.ram[usize::from(app.cpu.program_counter)],
            app.cpu.ram[usize::from(app.cpu.program_counter.wrapping_add(1))],
        ),
        Style::default().add_modifier(Modifier::BOLD),
    )];
    for i in 0..inner.height - 1 {
        let history = app
            .instruction_history
            .get(app.instruction_history.len().wrapping_sub(usize::from(i)));
        match history {
            Some((instruction, immediate)) => {
                lines.push(Line::from(disassemble(*instruction, *immediate)));
            }
            None => {
                lines.push(Line::from(""));
            }
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, rect);
}
