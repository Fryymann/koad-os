use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap, Gauge},
    Frame, Terminal,
};
use rusqlite::Connection;
use serde_json::Value;
use std::{env, io, time::Duration};

#[derive(Clone)]
struct Character {
    id: String,
    name: String,
    campaign: Option<String>,
    party: Option<String>,
    json: Value,
}

impl Character {
    fn get_primary_class(&self) -> String {
        self.json["classes"][0]["definition"]["name"].as_str().unwrap_or("Adventurer").to_string()
    }

    fn get_theme_color(&self) -> Color {
        match self.get_primary_class().to_lowercase().as_str() {
            "paladin" => Color::Rgb(255, 215, 0), // Gold
            "wizard" => Color::Rgb(100, 149, 237), // Cornflower Blue
            "rogue" => Color::Rgb(105, 105, 105), // Dim Gray
            "cleric" => Color::Rgb(245, 245, 245), // White Smoke
            "fighter" => Color::Rgb(178, 34, 34), // Firebrick
            "warlock" => Color::Rgb(138, 43, 226), // Blue Violet
            "druid" => Color::Rgb(34, 139, 34), // Forest Green
            "ranger" => Color::Rgb(46, 139, 87), // Sea Green
            "monk" => Color::Rgb(0, 206, 209), // Dark Turquoise
            "bard" => Color::Rgb(255, 105, 180), // Hot Pink
            "barbarian" => Color::Rgb(255, 69, 0), // Orange Red
            "sorcerer" => Color::Rgb(220, 20, 60), // Crimson
            _ => Color::White,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Tab {
    General,
    Stats,
    Skills,
    Features,
    Spells,
    Inventory,
}

#[derive(Copy, Clone, PartialEq)]
enum Pane {
    CharacterList,
    DetailList,
}

struct App {
    characters: Vec<Character>,
    list_state: ListState,
    active_tab: Tab,
    active_pane: Pane,
    feature_state: ListState,
    spell_state: ListState,
    inv_state: ListState,
    typewriter_tick: usize,
}

impl App {
    fn new(characters: Vec<Character>) -> App {
        let mut list_state = ListState::default();
        if !characters.is_empty() {
            list_state.select(Some(0));
        }
        App {
            characters,
            list_state,
            active_tab: Tab::General,
            active_pane: Pane::CharacterList,
            feature_state: ListState::default(),
            spell_state: ListState::default(),
            inv_state: ListState::default(),
            typewriter_tick: 0,
        }
    }

    fn on_tick(&mut self) {
        self.typewriter_tick += 15;
    }

    fn next(&mut self) {
        match self.active_pane {
            Pane::CharacterList => {
                let i = match self.list_state.selected() {
                    Some(i) => if i >= self.characters.len() - 1 { 0 } else { i + 1 },
                    None => 0,
                };
                self.list_state.select(Some(i));
                self.reset_typewriter();
                self.reset_sub_lists();
            },
            Pane::DetailList => {
                match self.active_tab {
                    Tab::Features => self.next_feature(),
                    Tab::Spells => self.next_spell(),
                    Tab::Inventory => self.next_inv(),
                    _ => {}
                }
                self.reset_typewriter();
            }
        }
    }

    fn previous(&mut self) {
        match self.active_pane {
            Pane::CharacterList => {
                let i = match self.list_state.selected() {
                    Some(i) => if i == 0 { self.characters.len() - 1 } else { i - 1 },
                    None => 0,
                };
                self.list_state.select(Some(i));
                self.reset_typewriter();
                self.reset_sub_lists();
            },
            Pane::DetailList => {
                match self.active_tab {
                    Tab::Features => self.prev_feature(),
                    Tab::Spells => self.prev_spell(),
                    Tab::Inventory => self.prev_inv(),
                    _ => {}
                }
                self.reset_typewriter();
            }
        }
    }

    fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::General => Tab::Stats,
            Tab::Stats => Tab::Skills,
            Tab::Skills => Tab::Features,
            Tab::Features => Tab::Spells,
            Tab::Spells => Tab::Inventory,
            Tab::Inventory => Tab::General,
        };
        self.reset_typewriter();
        self.reset_sub_lists();
    }

    fn reset_typewriter(&mut self) {
        self.typewriter_tick = 0;
    }

    fn reset_sub_lists(&mut self) {
        self.feature_state.select(Some(0));
        self.spell_state.select(Some(0));
        self.inv_state.select(Some(0));
    }

    fn next_feature(&mut self) {
        if let Some(c) = self.get_current_char() {
            let count = get_feature_count(&c);
            if count == 0 { return; }
            let i = match self.feature_state.selected() {
                Some(i) => if i >= count - 1 { 0 } else { i + 1 },
                None => 0,
            };
            self.feature_state.select(Some(i));
        }
    }
    fn prev_feature(&mut self) {
        if let Some(c) = self.get_current_char() {
            let count = get_feature_count(&c);
            if count == 0 { return; }
            let i = match self.feature_state.selected() {
                Some(i) => if i == 0 { count - 1 } else { i - 1 },
                None => 0,
            };
            self.feature_state.select(Some(i));
        }
    }

    fn next_spell(&mut self) {
        if let Some(c) = self.get_current_char() {
            let count = get_spell_count(&c);
            if count == 0 { return; }
            let i = match self.spell_state.selected() {
                Some(i) => if i >= count - 1 { 0 } else { i + 1 },
                None => 0,
            };
            self.spell_state.select(Some(i));
        }
    }
    fn prev_spell(&mut self) {
        if let Some(c) = self.get_current_char() {
            let count = get_spell_count(&c);
            if count == 0 { return; }
            let i = match self.spell_state.selected() {
                Some(i) => if i == 0 { count - 1 } else { i - 1 },
                None => 0,
            };
            self.spell_state.select(Some(i));
        }
    }

    fn next_inv(&mut self) {
        if let Some(c) = self.get_current_char() {
            let count = get_inv_count(&c);
            if count == 0 { return; }
            let i = match self.inv_state.selected() {
                Some(i) => if i >= count - 1 { 0 } else { i + 1 },
                None => 0,
            };
            self.inv_state.select(Some(i));
        }
    }
    fn prev_inv(&mut self) {
        if let Some(c) = self.get_current_char() {
            let count = get_inv_count(&c);
            if count == 0 { return; }
            let i = match self.inv_state.selected() {
                Some(i) => if i == 0 { count - 1 } else { i - 1 },
                None => 0,
            };
            self.inv_state.select(Some(i));
        }
    }

    fn get_current_char(&self) -> Option<Character> {
        self.list_state.selected().map(|i| self.characters[i].clone())
    }
}

fn get_feature_count(c: &Character) -> usize {
    let mut count = 0;
    if let Some(traits) = c.json["race"]["traits"].as_array() { count += traits.len(); }
    if let Some(classes) = c.json["classes"].as_array() {
        for cl in classes {
            if let Some(f) = cl["classFeatures"].as_array() { count += f.len(); }
        }
    }
    if let Some(feats) = c.json["feats"].as_array() { count += feats.len(); }
    count
}

fn get_spell_count(c: &Character) -> usize {
    let mut count = 0;
    if let Some(class_spells) = c.json["classSpells"].as_array() {
        for cs in class_spells {
            if let Some(s) = cs["spells"].as_array() { count += s.len(); }
        }
    }
    count
}

fn get_inv_count(c: &Character) -> usize {
    c.json["inventory"].as_array().map(|a| a.len()).unwrap_or(0)
}

fn load_characters() -> Result<Vec<Character>> {
    let home = env::var("HOME")?;
    let db_path = format!("{}/.koad-os/data/dnd_syncs.db", home);
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT dnd_beyond_id, name, full_json, campaign_name, party_name FROM character_syncs ORDER BY name")?;
    let char_iter = stmt.query_map([], |row| {
        let json_str: String = row.get(2)?;
        let json: Value = serde_json::from_str(&json_str).unwrap_or(Value::Null);
        Ok(Character {
            id: row.get(0)?,
            name: row.get(1)?,
            json,
            campaign: row.get(3)?,
            party: row.get(4)?,
        })
    })?;

    let mut chars = Vec::new();
    for c in char_iter {
        chars.push(c?);
    }
    Ok(chars)
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let characters = load_characters().context("Failed to load characters from DB")?;
    let mut app = App::new(characters);

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                        if app.active_pane == Pane::CharacterList {
                            app.active_pane = Pane::DetailList;
                            app.reset_sub_lists();
                            app.reset_typewriter();
                        }
                    },
                    KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => {
                        app.active_pane = Pane::CharacterList;
                        app.reset_typewriter();
                    },
                    _ => {}
                }
            }
        }
        app.on_tick();
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(f.size());

    let items: Vec<ListItem> = app
        .characters
        .iter()
        .map(|c| {
            let name_line = Line::from(vec![
                Span::styled(format!(" {} ", c.name), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]);
            let campaign = c.campaign.as_deref().unwrap_or("No Campaign");
            let meta_line = Line::from(vec![
                Span::styled(format!("   {}", campaign), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
            ]);
            ListItem::new(vec![name_line, meta_line, Line::from("")])
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(" Characters ")
        .border_style(if app.active_pane == Pane::CharacterList { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) });

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().bg(Color::Rgb(60, 20, 20)).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], &mut app.list_state);

    if let Some(char) = app.get_current_char() {
        render_details(f, chunks[1], app, &char);
    }
}

fn render_details(f: &mut Frame, area: Rect, app: &mut App, char: &Character) {
    let theme_color = char.get_theme_color();
    
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let titles = vec![" General ", " Stats ", " Skills ", " Features ", " Spells ", " Inventory "];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", char.name)).border_style(Style::default().fg(theme_color)))
        .select(match app.active_tab {
            Tab::General => 0,
            Tab::Stats => 1,
            Tab::Skills => 2,
            Tab::Features => 3,
            Tab::Spells => 4,
            Tab::Inventory => 5,
        })
        .highlight_style(Style::default().fg(theme_color).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, vertical_chunks[0]);

    let content_area = vertical_chunks[1];
    let content_block = Block::default().borders(Borders::ALL).border_style(
        if app.active_pane == Pane::DetailList { Style::default().fg(theme_color) } else { Style::default().fg(Color::DarkGray) }
    );

    match app.active_tab {
        Tab::General => render_general(f, content_area, content_block, char, app.typewriter_tick),
        Tab::Stats => render_stats(f, content_area, content_block, char),
        Tab::Skills => render_skills(f, content_area, content_block, char),
        Tab::Features => render_features(f, content_area, content_block, &mut app.feature_state, char, app.typewriter_tick),
        Tab::Spells => render_spells(f, content_area, content_block, &mut app.spell_state, char, app.typewriter_tick),
        Tab::Inventory => render_inventory(f, content_area, content_block, &mut app.inv_state, char, app.typewriter_tick),
    }
}

fn get_stat_mod(val: i64) -> i64 {
    (val - 10) / 2
}

fn apply_typewriter(text: &str, tick: usize) -> String {
    let len = text.chars().count();
    let end = if tick > len { len } else { tick };
    text.chars().take(end).collect()
}

fn render_general(f: &mut Frame, area: Rect, block: Block, char: &Character, tick: usize) {
    let data = &char.json;
    let race = data["race"]["fullName"].as_str().unwrap_or("Unknown");
    let hp_base = data["baseHitPoints"].as_i64().unwrap_or(0);
    let hp_removed = data["removedHitPoints"].as_i64().unwrap_or(0);
    let hp_temp = data["temporaryHitPoints"].as_i64().unwrap_or(0);
    let hp_current = hp_base - hp_removed + hp_temp;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    let hp_ratio = if hp_base > 0 { hp_current as f64 / hp_base as f64 } else { 0.0 };
    let gauge_color = if hp_ratio > 0.5 { Color::Green } else if hp_ratio > 0.2 { Color::Yellow } else { Color::Red };

    let hp_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Health Points "))
        .gauge_style(Style::default().fg(gauge_color).bg(Color::Rgb(20, 20, 20)).add_modifier(Modifier::BOLD))
        .ratio(hp_ratio.clamp(0.0, 1.0))
        .label(format!("{}/{} (+{} temp)", hp_current, hp_base, hp_temp));
    
    f.render_widget(hp_gauge, chunks[0]);

    let bio = data["notes"]["personalHistory"].as_str().unwrap_or("No history found.");
    let text = vec![
        Line::from(vec!["Campaign: ".bold().fg(Color::Yellow), char.campaign.as_deref().unwrap_or("None").into()]),
        Line::from(vec!["Party:    ".bold().fg(Color::Yellow), char.party.as_deref().unwrap_or("None").into()]),
        Line::from(vec!["Race:     ".bold(), race.into()]),
        Line::from(""),
        Line::from("Personal History:".bold().fg(Color::Cyan)),
        Line::from(apply_typewriter(bio, tick)),
    ];

    f.render_widget(Paragraph::new(text).block(block.title(" Info ")).wrap(Wrap { trim: true }), chunks[1]);
}

fn render_stats(f: &mut Frame, area: Rect, block: Block, char: &Character) {
    let data = &char.json;
    let stat_names = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
    let mut lines = Vec::new();

    if let Some(stats) = data["stats"].as_array() {
        for (i, s) in stats.iter().enumerate() {
            if i < 6 {
                let val = s["value"].as_i64().unwrap_or(0);
                lines.push(Line::from(vec![
                    Span::styled(format!("{}: ", stat_names[i]), Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{} ({:+})", val, get_stat_mod(val))),
                ]));
            }
        }
    }
    f.render_widget(Paragraph::new(lines).block(block.title(" Base Stats ")), area);
}

fn render_skills(f: &mut Frame, area: Rect, block: Block, char: &Character) {
    let data = &char.json;
    let skill_names = [
        ("Acrobatics", 2), ("Animal Handling", 5), ("Arcana", 4), ("Athletics", 1),
        ("Deception", 6), ("History", 4), ("Insight", 5), ("Intimidation", 6),
        ("Investigation", 4), ("Medicine", 5), ("Nature", 4), ("Perception", 5),
        ("Performance", 6), ("Persuasion", 6), ("Religion", 4), ("Sleight of Hand", 2),
        ("Stealth", 2), ("Survival", 5)
    ];

    let mut stats = [10i64; 7];
    if let Some(s_arr) = data["stats"].as_array() {
        for s in s_arr {
            if let (Some(id), Some(val)) = (s["id"].as_u64(), s["value"].as_i64()) {
                if id >= 1 && id <= 6 { stats[id as usize] = val; }
            }
        }
    }

    let mut lines = Vec::new();
    for (name, stat_id) in skill_names {
        let mod_val = get_stat_mod(stats[stat_id]);
        lines.push(Line::from(vec![
            Span::styled(format!("{:<16}", name), Style::default().fg(Color::White)),
            Span::raw(format!(" {:+}", mod_val)),
        ]));
    }

    f.render_widget(Paragraph::new(lines).block(block.title(" Skills (Base Modifiers) ")), area);
}

fn render_features(f: &mut Frame, area: Rect, block: Block, state: &mut ListState, char: &Character, tick: usize) {
    let mut features = Vec::new();
    let data = &char.json;

    if let Some(traits) = data["race"]["traits"].as_array() {
        for t in traits {
            features.push((
                t["definition"]["name"].as_str().unwrap_or("Unknown Trait").to_string(),
                t["definition"]["description"].as_str().unwrap_or("No description.").to_string(),
                "Race".to_string()
            ));
        }
    }
    if let Some(classes) = data["classes"].as_array() {
        for c in classes {
            let class_name = c["definition"]["name"].as_str().unwrap_or("Class");
            if let Some(fs) = c["classFeatures"].as_array() {
                for f in fs {
                    features.push((
                        f["definition"]["name"].as_str().unwrap_or("Unknown Feature").to_string(),
                        f["definition"]["description"].as_str().unwrap_or("No description.").to_string(),
                        class_name.to_string()
                    ));
                }
            }
        }
    }
    if let Some(feats) = data["feats"].as_array() {
        for ft in feats {
            features.push((
                ft["definition"]["name"].as_str().unwrap_or("Unknown Feat").to_string(),
                ft["definition"]["description"].as_str().unwrap_or("No description.").to_string(),
                "Feat".to_string()
            ));
        }
    }

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let items: Vec<ListItem> = features.iter().map(|(n, _, src)| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("[{}] ", src), Style::default().fg(Color::DarkGray)),
            Span::raw(n),
        ]))
    }).collect();

    let list = List::new(items)
        .block(block.title(" Features List "))
        .highlight_style(Style::default().bg(Color::Rgb(40, 40, 40)).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, split[0], state);

    let desc = if let Some(i) = state.selected() {
        features.get(i).map(|f| f.1.clone()).unwrap_or_default()
    } else {
        "Select a feature to view description.".to_string()
    };

    let clean_desc = clean_html(&desc);
    f.render_widget(Paragraph::new(apply_typewriter(&clean_desc, tick)).block(Block::default().borders(Borders::ALL).title(" Description ")).wrap(Wrap { trim: true }), split[1]);
}

fn render_spells(f: &mut Frame, area: Rect, block: Block, state: &mut ListState, char: &Character, tick: usize) {
    let mut all_spells = Vec::new();
    let data = &char.json;

    if let Some(class_spells) = data["classSpells"].as_array() {
        for cs in class_spells {
            if let Some(spells) = cs["spells"].as_array() {
                for s in spells {
                    all_spells.push(s.clone());
                }
            }
        }
    }

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let items: Vec<ListItem> = all_spells.iter().map(|s| {
        let name = s["definition"]["name"].as_str().unwrap_or("Unknown");
        let level = s["definition"]["level"].as_i64().unwrap_or(0);
        ListItem::new(Line::from(vec![
            Span::styled(format!("[Lvl {}] ", level), Style::default().fg(Color::Cyan)),
            Span::raw(name),
        ]))
    }).collect();

    let list = List::new(items)
        .block(block.title(" Spellbook "))
        .highlight_style(Style::default().bg(Color::Rgb(40, 40, 40)).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, split[0], state);

    let desc = if let Some(i) = state.selected() {
        if let Some(s) = all_spells.get(i) {
            let d = s["definition"]["description"].as_str().unwrap_or("No description.");
            let time = s["definition"]["castingTime"]["castingTimeInterval"].as_i64().unwrap_or(0);
            let unit = s["definition"]["castingTime"]["castingTimeUnit"].as_str().unwrap_or("");
            format!("Casting Time: {} {}\n\n{}", time, unit, d)
        } else { "".into() }
    } else { "Select a spell.".into() };

    let clean_desc = clean_html(&desc);
    f.render_widget(Paragraph::new(apply_typewriter(&clean_desc, tick)).block(Block::default().borders(Borders::ALL).title(" Spell Details ")).wrap(Wrap { trim: true }), split[1]);
}

fn render_inventory(f: &mut Frame, area: Rect, block: Block, state: &mut ListState, char: &Character, tick: usize) {
    let mut items_vec = Vec::new();
    let data = &char.json;

    if let Some(inv) = data["inventory"].as_array() {
        for item in inv {
            items_vec.push(item.clone());
        }
    }

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let items: Vec<ListItem> = items_vec.iter().map(|item| {
        let name = item["definition"]["name"].as_str().unwrap_or("Unknown");
        let qty = item["quantity"].as_i64().unwrap_or(1);
        let equipped = item["equipped"].as_bool().unwrap_or(false);
        let status = if equipped { "[E] " } else { "    " };
        ListItem::new(Line::from(vec![
            Span::styled(status, Style::default().fg(Color::Green)),
            Span::raw(format!("{} (x{})", name, qty)),
        ]))
    }).collect();

    let list = List::new(items)
        .block(block.title(" Inventory "))
        .highlight_style(Style::default().bg(Color::Rgb(40, 40, 40)).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, split[0], state);

    let desc = if let Some(i) = state.selected() {
        if let Some(it) = items_vec.get(i) {
            let d = it["definition"]["description"].as_str().unwrap_or("No description.");
            let weight = it["definition"]["weight"].as_f64().unwrap_or(0.0);
            format!("Weight: {} lbs\n\n{}", weight, d)
        } else { "".into() }
    } else { "Select an item.".into() };

    let clean_desc = clean_html(&desc);
    f.render_widget(Paragraph::new(apply_typewriter(&clean_desc, tick)).block(Block::default().borders(Borders::ALL).title(" Item Details ")).wrap(Wrap { trim: true }), split[1]);
}

fn clean_html(text: &str) -> String {
    text.replace("<p>", "").replace("</p>", "\n").replace("<br>", "\n").replace("<br />", "\n")
        .replace("<strong>", "").replace("</strong>", "")
        .replace("<em>", "").replace("</em>", "")
        .replace("<ul>", "").replace("</ul>", "")
        .replace("<li>", " • ").replace("</li>", "\n")
        .replace("&nbsp;", " ")
        .replace("&rsquo;", "'")
        .replace("&lsquo;", "'")
        .replace("&ldquo;", "\"")
        .replace("&rdquo;", "\"")
        .replace("<hr />", "---")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_stat_mod() {
        assert_eq!(get_stat_mod(10), 0);
        assert_eq!(get_stat_mod(11), 0);
        assert_eq!(get_stat_mod(12), 1);
        assert_eq!(get_stat_mod(8), -1);
        assert_eq!(get_stat_mod(20), 5);
    }

    #[test]
    fn test_clean_html() {
        let input = "<p>Hello <strong>World</strong>!</p>";
        let output = clean_html(input);
        assert!(output.contains("Hello World!"));
        assert!(!output.contains("<p>"));
        assert!(!output.contains("<strong>"));
    }

    #[test]
    fn test_apply_typewriter() {
        let text = "Hello World";
        assert_eq!(apply_typewriter(text, 0), "");
        assert_eq!(apply_typewriter(text, 5), "Hello");
        assert_eq!(apply_typewriter(text, 50), "Hello World");
    }
}
