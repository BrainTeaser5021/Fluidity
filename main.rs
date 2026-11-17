use freedesktop_desktop_entry::{default_paths, Iter};
use iced::widget::{column, container, scrollable, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Task};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::Application;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::process::Command;

#[derive(Debug, Clone)]
struct AppEntry {
    name: String,
    exec: String,
    categories: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    QueryChanged(String),
    LaunchApp(String),
    AskAi(String),
    AiResponse(String),
}

struct FluidityLauncher {
    query: String,
    apps: Vec<AppEntry>,
    filtered_apps: Vec<AppEntry>,
    ai_answer: Option<String>,
    is_thinking: bool,
}

/// Index ONLY user-launchable applications; explicitly exclude system files and settings panels.
fn load_user_apps() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    for path in Iter::new(default_paths()) {
        if let Ok(bytes) = std::fs::read_to_string(&path) {
            if let Ok(entry) = freedesktop_desktop_entry::DesktopEntry::from_str(&path, &bytes, None::<&[&str]>) {
                if entry.no_display() || entry.hidden() || entry.type_() != Some("Application") {
                    continue;
                }
                let categories: Vec<String> = entry
                    .categories()
                    .unwrap_or_default()
                    .into_iter()
                    .map(String::from)
                    .collect();

                // Exclude system settings, control panels, and administrative tools
                let is_system_or_settings = categories.iter().any(|c| {
                    matches!(
                        c.as_str(),
                        "Settings" | "DesktopSettings" | "HardwareSettings" | "System" | "X-KDE-settings"
                    )
                });
                if is_system_or_settings {
                    continue;
                }

                if let (Some(name), Some(exec)) = (entry.name(&[]), entry.exec()) {
                    apps.push(AppEntry {
                        name: name.to_string(),
                        exec: exec.to_string(),
                        categories,
                    });
                }
            }
        }
    }
    apps.sort_by(|a, b| a.name.cmp(&b.name));
    apps
}

fn looks_like_question(q: &str, app_hits: usize) -> bool {
    let trimmed = q.trim();
    if trimmed.is_empty() {
        return false;
    }
    trimmed.ends_with('?')
        || (trimmed.split_whitespace().count() >= 3 && app_hits == 0)
}

impl Application for FluidityLauncher {
    type Message = Message;
    type Flags = ();
    type Theme = iced::Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Task<Message>) {
        let apps = load_user_apps();
        let filtered_apps = apps.iter().take(6).cloned().collect();
        (
            Self {
                query: String::new(),
                apps,
                filtered_apps,
                ai_answer: None,
                is_thinking: false,
            },
            Task::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("fluidity-launcher")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::QueryChanged(q) => {
                self.query = q.clone();
                self.ai_answer = None;

                let mut matcher = Matcher::new(Config::DEFAULT);
                let pattern = Pattern::parse(&q, CaseMatching::Ignore, Normalization::Smart);
                let mut scored: Vec<(u32, AppEntry)> = self
                    .apps
                    .iter()
                    .filter_map(|app| {
                        let mut buf = Vec::new();
                        let hay = Utf32Str::new(&app.name, &mut buf);
                        pattern.score(hay, &mut matcher).map(|s| (s, app.clone()))
                    })
                    .collect();
                scored.sort_by(|a, b| b.0.cmp(&a.0));
                self.filtered_apps = scored.into_iter().take(6).map(|(_, a)| a).collect();

                if looks_like_question(&self.query, self.filtered_apps.len()) {
                    return Task::perform(async move { q }, Message::AskAi);
                }
                Task::none()
            }
            Message::LaunchApp(exec) => {
                let clean_cmd = exec.split_whitespace().next().unwrap_or(&exec);
                let _ = Command::new(clean_cmd).spawn();
                std::process::exit(0);
            }
            Message::AskAi(question) => {
                self.is_thinking = true;
                Task::perform(
                    async move {
                        // Lazy-load Qwen2.5-0.5B-Instruct (or 1.5B) via mistralrs here,
                        // run concise inference, and drop model after idle timeout to minimize RAM.
                        format!("Local AI (Qwen2.5-0.5B-Instruct) ready for: \"{}\"", question)
                    },
                    Message::AiResponse,
                )
            }
            Message::AiResponse(ans) => {
                self.is_thinking = false;
                self.ai_answer = Some(ans);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let input = text_input("Search apps or ask anything...", &self.query)
            .on_input(Message::QueryChanged)
            .padding(14);

        let mut results = column![].spacing(8);
        for app in &self.filtered_apps {
            let cmd = app.exec.clone();
            results = results.push(
                iced::widget::button(text(&app.name).size(16))
                    .on_press(Message::LaunchApp(cmd))
                    .width(Length::Fill),
            );
        }

        if self.is_thinking {
            results = results.push(text("Thinking locally...").size(14));
        } else if let Some(ans) = &self.ai_answer {
            results = results.push(text(ans).size(15));
        }

        // Liquid-glass translucent container (KWin blurs the layer-shell background behind it)
        container(
            column![input, scrollable(results)]
                .spacing(12)
                .align_x(Alignment::Start),
        )
        .padding(20)
        .width(Length::Fixed(640.0))
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.08, 0.11, 0.16, 0.62))),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.22),
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}

fn main() -> Result<(), iced_layershell::Error> {
    FluidityLauncher::run(Settings {
        layer_settings: LayerShellSettings {
            size: Some((680, 420)),
            layer: Layer::Overlay,
            anchor: Anchor::empty(),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            ..Default::default()
        },
        ..Default::default()
    })
}
