#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use rodio::source::{SineWave, Source};
use chrono::Datelike;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum TimerMode {
    Work,
    Break,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimerStatus {
    Ready,
    Focusing,
    Paused,
    Break,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ThemePreset {
    SunsetGlow,
    ForestWhisper,
    DeepCosmos,
    Cyberpunk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum AmbientStyle {
    None,
    SpacePad,
    BinauralBeats,
    WhiteNoise,
    BrownNoise,
    LofiBeat,
    CozyRain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum NoiseStyle {
    White,
    Brown,
    Rain,
}

#[derive(Clone, Copy)]
struct Xorshift {
    state: u32,
}

impl Xorshift {
    fn new() -> Self {
        Self { state: 123456789 }
    }

    fn next_f32(&mut self) -> f32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

#[derive(Clone, Copy)]
struct NoiseSource {
    style: NoiseStyle,
    xorshift: Xorshift,
    last_brown: f32,
}

impl NoiseSource {
    fn new(style: NoiseStyle) -> Self {
        Self {
            style,
            xorshift: Xorshift::new(),
            last_brown: 0.0,
        }
    }
}

impl Iterator for NoiseSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let white = self.xorshift.next_f32();
        match self.style {
            NoiseStyle::White => Some(white),
            NoiseStyle::Brown => {
                let brown = (self.last_brown + (0.05 * white)) / 1.02;
                self.last_brown = brown;
                Some(brown * 5.0)
            }
            NoiseStyle::Rain => {
                // Rain is synthesized by combining:
                // 1. A deep base low-frequency rumble (filtered Brown noise)
                // 2. High-frequency raindrop splatters (random transient clicks)
                let brown = (self.last_brown + (0.05 * white)) / 1.02;
                self.last_brown = brown;
                
                let rand_val = self.xorshift.next_f32();
                let splatter = if rand_val > 0.9997 {
                    self.xorshift.next_f32() * 0.4
                } else {
                    0.0
                };
                Some(brown * 3.5 + splatter)
            }
        }
    }
}

impl Source for NoiseSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TodoItem {
    id: String,
    title: String,
    completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct HistoryEntry {
    id: String,
    timestamp: chrono::DateTime<chrono::Local>,
    task_name: String,
    duration_mins: u32,
    notes: String,
    mode: TimerMode,
    #[serde(default = "default_category")]
    category: String,
}

fn default_category() -> String {
    "Studying".to_string()
}

struct AudioPlayer {
    _stream: Option<rodio::OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
}

impl AudioPlayer {
    fn new() -> Self {
        match rodio::OutputStream::try_default() {
            Ok((stream, stream_handle)) => Self {
                _stream: Some(stream),
                stream_handle: Some(stream_handle),
            },
            Err(e) => {
                eprintln!("Failed to initialize audio output stream: {:?}", e);
                Self {
                    _stream: None,
                    stream_handle: None,
                }
            }
        }
    }

    fn play_work_complete(&self) {
        if let Some(ref handle) = self.stream_handle {
            if let Ok(sink) = rodio::Sink::try_new(handle) {
                // Ascending chime: E5 (659.25 Hz), A5 (880.00 Hz), C#6 (1109.73 Hz)
                let notes = [659.25, 880.00, 1109.73];
                for &freq in &notes {
                    let source = SineWave::new(freq)
                        .take_duration(std::time::Duration::from_millis(150))
                        .amplify(0.1);
                    sink.append(source);
                }
                sink.detach();
            }
        }
    }

    fn play_break_complete(&self) {
        if let Some(ref handle) = self.stream_handle {
            if let Ok(sink) = rodio::Sink::try_new(handle) {
                // Soft warm chime: G4 (392.00 Hz) then C5 (523.25 Hz)
                let source1 = SineWave::new(392.00)
                    .take_duration(std::time::Duration::from_millis(200))
                    .amplify(0.12);
                let source2 = SineWave::new(523.25)
                    .take_duration(std::time::Duration::from_millis(400))
                    .amplify(0.12);
                sink.append(source1);
                sink.append(source2);
                sink.detach();
            }
        }
    }

    fn play_ambient_chord(&self, chord_index: usize) -> Vec<rodio::Sink> {
        let mut spawned_sinks = Vec::new();
        if let Some(ref handle) = self.stream_handle {
            // Harmonious pentatonic progressions
            let chords = [
                vec![130.81, 196.00, 246.94, 329.63], // C3, G3, B3, E4
                vec![110.00, 164.81, 196.00, 261.63], // A2, E3, G3, C4
                vec![87.31, 130.81, 164.81, 220.00],  // F2, C3, E3, A3
                vec![98.00, 146.83, 174.61, 246.94],  // G2, D3, F3, B3
            ];
            let freqs = &chords[chord_index % chords.len()];
            for &freq in freqs {
                if let Ok(sink) = rodio::Sink::try_new(handle) {
                    let source = SineWave::new(freq)
                        .take_duration(std::time::Duration::from_secs(8))
                        .fade_in(std::time::Duration::from_secs(3))
                        .amplify(0.025); // Warm ambient volume (amplified)
                    sink.append(source);
                    spawned_sinks.push(sink);
                }
            }
        }
        spawned_sinks
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    theme: ThemePreset,
    categories: Vec<String>,
    active_category: String,
    ambient_style: AmbientStyle,
    task_queue: Vec<TodoItem>,
    work_duration_mins: u32,
    break_duration_mins: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemePreset::SunsetGlow,
            categories: vec![
                "Studying".to_string(),
                "Coding".to_string(),
                "Writing".to_string(),
                "Research".to_string(),
                "Design".to_string(),
            ],
            active_category: "Studying".to_string(),
            ambient_style: AmbientStyle::None,
            task_queue: Vec::new(),
            work_duration_mins: 25,
            break_duration_mins: 5,
        }
    }
}

struct ConfigManager {
    file_path: PathBuf,
}

impl ConfigManager {
    fn new() -> Self {
        let mut path = dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        path.push(".focus-flow");
        let _ = std::fs::create_dir_all(&path);
        path.push("config.json");
        Self { file_path: path }
    }

    fn load_config(&self) -> AppConfig {
        if !self.file_path.exists() {
            return AppConfig::default();
        }
        match std::fs::read_to_string(&self.file_path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|_| AppConfig::default()),
            Err(_) => AppConfig::default(),
        }
    }

    fn save_config(&self, config: &AppConfig) {
        if let Ok(contents) = serde_json::to_string_pretty(config) {
            let _ = std::fs::write(&self.file_path, contents);
        }
    }
}

struct HistoryManager {
    file_path: PathBuf,
}

impl HistoryManager {
    fn new() -> Self {
        let mut path = dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        path.push(".focus-flow");
        let _ = std::fs::create_dir_all(&path);
        path.push("history.json");
        Self { file_path: path }
    }

    fn load_history(&self) -> Vec<HistoryEntry> {
        if !self.file_path.exists() {
            return Vec::new();
        }
        match std::fs::read_to_string(&self.file_path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    fn save_history(&self, history: &[HistoryEntry]) -> Result<(), std::io::Error> {
        let contents = serde_json::to_string_pretty(history)?;
        std::fs::write(&self.file_path, contents)?;
        Ok(())
    }

    fn clear_history(&self) -> Result<(), std::io::Error> {
        if self.file_path.exists() {
            std::fs::remove_file(&self.file_path)?;
        }
        Ok(())
    }
}

struct FocusFlowApp {
    mode: TimerMode,
    status: TimerStatus,
    is_running: bool,
    time_remaining: f64,
    total_duration: f64,

    work_duration_mins: u32,
    break_duration_mins: u32,

    task_name: String,
    session_notes: String,

    history: Vec<HistoryEntry>,
    history_manager: HistoryManager,
    show_clear_confirm: bool,

    // Visual animation states
    hue_accumulator: f32,
    breathing_accumulator: f32,
    last_tick: std::time::Instant,

    // Audio
    audio_player: AudioPlayer,

    // PREMIUM CONFIG & STATE:
    config_manager: ConfigManager,
    theme: ThemePreset,
    categories: Vec<String>,
    active_category: String,
    new_tag_input: String,

    ambient_style: AmbientStyle,
    ambient_timer: f32,
    ambient_chord_index: usize,
    ambient_sink: Option<rodio::Sink>,

    task_queue: Vec<TodoItem>,
    new_todo_input: String,

    compact_mode: bool,
    last_compact_state: bool,
    dynamic_sinks: Vec<rodio::Sink>,
}

impl FocusFlowApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Setup premium rounded visuals
        cc.egui_ctx.global_style_mut(|style| {
            style.visuals.window_corner_radius = egui::CornerRadius::same(12);
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
        });

        let history_manager = HistoryManager::new();
        let history = history_manager.load_history();

        let config_manager = ConfigManager::new();
        let config = config_manager.load_config();

        let time_remaining = config.work_duration_mins as f64 * 60.0;
        let audio_player = AudioPlayer::new();

        let mut app = Self {
            mode: TimerMode::Work,
            status: TimerStatus::Ready,
            is_running: false,
            time_remaining,
            total_duration: time_remaining,

            work_duration_mins: config.work_duration_mins,
            break_duration_mins: config.break_duration_mins,

            task_name: String::new(),
            session_notes: String::new(),

            history,
            history_manager,
            show_clear_confirm: false,

            hue_accumulator: 0.0,
            breathing_accumulator: 0.0,
            last_tick: std::time::Instant::now(),

            audio_player,

            config_manager,
            theme: config.theme,
            categories: config.categories,
            active_category: config.active_category,
            new_tag_input: String::new(),

            ambient_style: config.ambient_style,
            ambient_timer: 0.0,
            ambient_chord_index: 0,
            ambient_sink: None,

            task_queue: config.task_queue,
            new_todo_input: String::new(),

            compact_mode: false,
            last_compact_state: false,
            dynamic_sinks: Vec::new(),
        };

        // Initialize ambient music state if loaded configuration is set
        app.update_ambient_sound();

        // Spawn background System Tray (hidden icons) helper script
        #[cfg(target_os = "windows")]
        {
            let mut helper_path = std::path::PathBuf::from("tray_helper.ps1");
            if !helper_path.exists() {
                if let Some(local_data) = dirs::data_local_dir() {
                    helper_path = local_data.join("Programs").join("Focus Flow").join("tray_helper.ps1");
                }
            }
            if helper_path.exists() {
                let mut cmd = std::process::Command::new("powershell");
                cmd.arg("-WindowStyle")
                    .arg("Hidden")
                    .arg("-ExecutionPolicy")
                    .arg("Bypass")
                    .arg("-File")
                    .arg(helper_path);
                cmd.creation_flags(CREATE_NO_WINDOW);
                cmd.spawn().ok();
            }
        }

        #[cfg(target_os = "macos")]
        {
            let is_bundled = if let Ok(exe_path) = std::env::current_exe() {
                exe_path.to_string_lossy().contains(".app/Contents/MacOS")
            } else {
                false
            };
            let bundle_id = if is_bundled {
                "com.focusflow.pomodoro-timer"
            } else {
                "com.apple.Terminal"
            };
            if let Err(e) = mac_notification_sys::set_application(bundle_id) {
                eprintln!("Failed to set notification application to {}: {:?}", bundle_id, e);
            }
        }

        Self::show_notification("Focus Flow", "Time to lock in!");
        app
    }

    fn persist_config(&self) {
        let config = AppConfig {
            theme: self.theme,
            categories: self.categories.clone(),
            active_category: self.active_category.clone(),
            ambient_style: self.ambient_style,
            task_queue: self.task_queue.clone(),
            work_duration_mins: self.work_duration_mins,
            break_duration_mins: self.break_duration_mins,
        };
        self.config_manager.save_config(&config);
    }

    fn update_ambient_sound(&mut self) {
        // Explicitly and completely halt the previous stream to prevent stuck background audio!
        if let Some(ref sink) = self.ambient_sink {
            sink.stop();
        }
        self.ambient_sink = None;

        // Clean up and halt all active dynamic note/beat sinks
        for sink in &self.dynamic_sinks {
            sink.stop();
        }
        self.dynamic_sinks.clear();

        if let Some(ref handle) = self.audio_player.stream_handle {
            match self.ambient_style {
                AmbientStyle::None | AmbientStyle::SpacePad | AmbientStyle::LofiBeat => {
                    // Sequenced beat players will tick in the update loop rather than static infinite sinks
                }
                AmbientStyle::BinauralBeats => {
                    if let Ok(sink) = rodio::Sink::try_new(handle) {
                        let s1 = SineWave::new(120.0).amplify(0.015);
                        let s2 = SineWave::new(125.0).amplify(0.015);
                        let mixed = s1.mix(s2);
                        sink.append(mixed);
                        self.ambient_sink = Some(sink);
                    }
                }
                AmbientStyle::WhiteNoise => {
                    if let Ok(sink) = rodio::Sink::try_new(handle) {
                        let source = NoiseSource::new(NoiseStyle::White).amplify(0.012);
                        sink.append(source);
                        self.ambient_sink = Some(sink);
                    }
                }
                AmbientStyle::BrownNoise => {
                    if let Ok(sink) = rodio::Sink::try_new(handle) {
                        let source = NoiseSource::new(NoiseStyle::Brown).amplify(0.025);
                        sink.append(source);
                        self.ambient_sink = Some(sink);
                    }
                }
                AmbientStyle::CozyRain => {
                    if let Ok(sink) = rodio::Sink::try_new(handle) {
                        let source = NoiseSource::new(NoiseStyle::Rain).amplify(0.025);
                        sink.append(source);
                        self.ambient_sink = Some(sink);
                    }
                }
            }
        }
    }

    fn show_notification(title: &str, message: &str) {
        #[cfg(target_os = "windows")]
        {
            let script = format!(
                r#"[void] [System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); $notification = New-Object System.Windows.Forms.NotifyIcon; $notification.Icon = [System.Drawing.SystemIcons]::Information; $notification.BalloonTipTitle = '{}'; $notification.BalloonTipText = '{}'; $notification.Visible = $true; $notification.ShowBalloonTip(5000);"#,
                title.replace('\'', "''"), message.replace('\'', "''")
            );
            let mut cmd = std::process::Command::new("powershell");
            cmd.arg("-Command").arg(script);
            cmd.creation_flags(CREATE_NO_WINDOW);
            cmd.spawn().ok();
        }
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            let title = title.to_string();
            let message = message.to_string();
            std::thread::spawn(move || {
                let mut notification = notify_rust::Notification::new();
                notification.summary(&title).body(&message);
                #[cfg(target_os = "macos")]
                {
                    notification.sound_name("Submarine");
                    if let Ok(mut path) = std::env::current_dir() {
                        path.push("logo.png");
                        if path.exists() {
                            if let Some(path_str) = path.to_str() {
                                notification.icon(path_str);
                            }
                        }
                    }
                }
                let _ = notification.show();
            });
        }
    }

    fn handle_timer_complete(&mut self) {
        match self.mode {
            TimerMode::Work => {
                let duration = self.work_duration_mins;
                self.save_session_to_history(duration);
                self.audio_player.play_work_complete();
                Self::show_notification("Focus Session Complete", "Great job! Time to take a breather and step away.");

                // Switch to break
                self.mode = TimerMode::Break;
                self.time_remaining = self.break_duration_mins as f64 * 60.0;
                self.total_duration = self.time_remaining;
                self.session_notes.clear();
                self.is_running = true;
                self.status = TimerStatus::Break;
            }
            TimerMode::Break => {
                self.audio_player.play_break_complete();
                Self::show_notification("Break's Over!", "Ready to flow? Let's lock back in.");

                // Switch to work
                self.mode = TimerMode::Work;
                self.time_remaining = self.work_duration_mins as f64 * 60.0;
                self.total_duration = self.time_remaining;
                self.is_running = false;
                self.status = TimerStatus::Ready;
            }
        }
    }

    fn save_session_to_history(&mut self, duration_mins: u32) {
        let task = if self.task_name.trim().is_empty() {
            "Unnamed Focus Session".to_string()
        } else {
            self.task_name.trim().to_string()
        };

        let notes = if self.session_notes.trim().is_empty() {
            "No notes recorded.".to_string()
        } else {
            self.session_notes.trim().to_string()
        };

        let entry = HistoryEntry {
            id: format!("{}", chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)),
            timestamp: chrono::Local::now(),
            task_name: task,
            duration_mins,
            notes,
            mode: TimerMode::Work,
            category: self.active_category.clone(),
        };

        let old_streak = self.calculate_daily_streak();
        
        self.history.push(entry);
        let _ = self.history_manager.save_history(&self.history);
        
        let new_streak = self.calculate_daily_streak();
        if new_streak > old_streak {
            Self::show_notification(
                "Streak Extended!",
                &format!("Amazing! You are now on a {} Day Focus Streak!", new_streak)
            );
        }
    }

    fn calculate_daily_streak(&self) -> u32 {
        if self.history.is_empty() {
            return 0;
        }
        
        let dates: std::collections::BTreeSet<chrono::NaiveDate> = self.history.iter()
            .map(|entry| entry.timestamp.date_naive())
            .collect();
            
        let today = chrono::Local::now().date_naive();
        let yesterday = today - chrono::TimeDelta::days(1);
        
        let mut current_date = if dates.contains(&today) {
            today
        } else if dates.contains(&yesterday) {
            yesterday
        } else {
            return 0;
        };
        
        let mut streak = 0;
        while dates.contains(&current_date) {
            streak += 1;
            if let Some(prev) = current_date.checked_sub_days(chrono::Days::new(1)) {
                current_date = prev;
            } else {
                break;
            }
        }
        streak
    }

    fn get_breathing_value(&self) -> f32 {
        let t = self.breathing_accumulator;
        if t < 4.0 {
            // Inhale (0.0 -> 1.0)
            t / 4.0
        } else if t < 8.0 {
            // Hold In (1.0)
            1.0
        } else if t < 12.0 {
            // Exhale (1.0 -> 0.0)
            1.0 - (t - 8.0) / 4.0
        } else {
            // Hold Out (0.0)
            0.0
        }
    }

    fn get_breathing_text(&self) -> &'static str {
        let t = self.breathing_accumulator;
        if t < 4.0 {
            "Breathe in..."
        } else if t < 8.0 {
            "Hold..."
        } else if t < 12.0 {
            "Breathe out..."
        } else {
            "Hold..."
        }
    }

    fn hsl_to_color32(h: f32, s: f32, l: f32) -> egui::Color32 {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
        let m = l - c / 2.0;
        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        egui::Color32::from_rgb(
            ((r + m) * 255.0).round().clamp(0.0, 255.0) as u8,
            ((g + m) * 255.0).round().clamp(0.0, 255.0) as u8,
            ((b + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        )
    }

    fn draw_badge(ui: &mut egui::Ui, text: &str, bg_color: egui::Color32, text_color: egui::Color32) {
        let padding = egui::vec2(10.0, 4.0);
        let font_id = egui::FontId::proportional(11.0);
        let galley = ui.painter().layout_no_wrap(text.to_string(), font_id.clone(), text_color);
        let rect_size = galley.size() + padding * 2.0;

        let (rect, _) = ui.allocate_at_least(rect_size, egui::Sense::hover());
        ui.painter().rect_filled(rect, 4.0, bg_color);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            font_id,
            text_color,
        );
    }

    fn get_theme_accent_color(&self) -> egui::Color32 {
        if self.mode == TimerMode::Work {
            match self.theme {
                ThemePreset::SunsetGlow => egui::Color32::from_rgb(255, 110, 80),   // Sunset Orange
                ThemePreset::ForestWhisper => egui::Color32::from_rgb(80, 220, 140), // Emerald
                ThemePreset::DeepCosmos => egui::Color32::from_rgb(180, 100, 255),   // Nebula Purple
                ThemePreset::Cyberpunk => egui::Color32::from_rgb(255, 50, 150),    // Neon Pink
            }
        } else {
            match self.theme {
                ThemePreset::SunsetGlow => egui::Color32::from_rgb(240, 200, 80),   // Warm Gold
                ThemePreset::ForestWhisper => egui::Color32::from_rgb(140, 200, 160), // Calming Sage
                ThemePreset::DeepCosmos => egui::Color32::from_rgb(100, 140, 255),   // Space Indigo
                ThemePreset::Cyberpunk => egui::Color32::from_rgb(0, 240, 255),     // Electric Cyan
            }
        }
    }

    fn get_theme_bg_color(&self) -> egui::Color32 {
        if self.mode == TimerMode::Work {
            let base_hue = match self.theme {
                ThemePreset::SunsetGlow => 15.0,
                ThemePreset::ForestWhisper => 140.0,
                ThemePreset::DeepCosmos => 275.0,
                ThemePreset::Cyberpunk => 330.0,
            };
            let h = (base_hue + self.hue_accumulator) % 360.0;
            Self::hsl_to_color32(h, 0.12, 0.08)
        } else {
            let breathing_val = self.get_breathing_value();
            let l = 0.06 + 0.04 * breathing_val;
            let base_hue = match self.theme {
                ThemePreset::SunsetGlow => 45.0,
                ThemePreset::ForestWhisper => 150.0,
                ThemePreset::DeepCosmos => 230.0,
                ThemePreset::Cyberpunk => 190.0,
            };
            Self::hsl_to_color32(base_hue, 0.15, l)
        }
    }

    fn handle_skip(&mut self) {
        if self.mode == TimerMode::Work {
            let elapsed = self.total_duration - self.time_remaining;
            let elapsed_mins = (elapsed / 60.0).round() as u32;
            if elapsed_mins >= 1 {
                self.save_session_to_history(elapsed_mins);
            }

            self.audio_player.play_work_complete();
            self.mode = TimerMode::Break;
            self.time_remaining = self.break_duration_mins as f64 * 60.0;
            self.total_duration = self.time_remaining;
            self.session_notes.clear();
            self.is_running = true;
            self.status = TimerStatus::Break;
        } else {
            self.audio_player.play_break_complete();
            self.mode = TimerMode::Work;
            self.time_remaining = self.work_duration_mins as f64 * 60.0;
            self.total_duration = self.time_remaining;
            self.is_running = false;
            self.status = TimerStatus::Ready;
        }
    }

    fn render_analytics(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Productivity Analytics").strong().color(egui::Color32::WHITE));
            ui.add_space(6.0);

            // 1. Gather stats for the last 7 days
            let mut day_totals = [0u32; 7];
            let today = chrono::Local::now().date_naive();
            let mut dates = Vec::new();
            for i in (0..7).rev() {
                dates.push(today - chrono::Days::new(i));
            }

            for entry in &self.history {
                if entry.mode == TimerMode::Work {
                    let entry_date = entry.timestamp.date_naive();
                    if let Some(pos) = dates.iter().position(|&d| d == entry_date) {
                        day_totals[pos] += entry.duration_mins;
                    }
                }
            }

            // 2. Render Bar Chart
            let (response, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), 90.0), egui::Sense::hover());
            let rect = response.rect;

            // Base line
            let baseline_y = rect.max.y - 15.0;
            painter.line_segment(
                [egui::pos2(rect.min.x, baseline_y), egui::pos2(rect.max.x, baseline_y)],
                egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30))
            );

            let num_bars = 7;
            let bar_spacing = rect.width() / num_bars as f32;
            let bar_width = bar_spacing * 0.55;

            let max_val = *day_totals.iter().max().unwrap_or(&0) as f32;
            let max_val = if max_val == 0.0 { 60.0 } else { max_val };

            for i in 0..num_bars {
                let val = day_totals[i] as f32;
                let ratio = val / max_val;
                let bar_height = ratio * (rect.height() - 32.0);

                let bar_center_x = rect.min.x + (i as f32 * bar_spacing) + bar_spacing * 0.5;
                let bar_rect = egui::Rect::from_min_max(
                    egui::pos2(bar_center_x - bar_width * 0.5, baseline_y - bar_height),
                    egui::pos2(bar_center_x + bar_width * 0.5, baseline_y)
                );

                let bar_color = if val > 0.0 {
                    self.get_theme_accent_color()
                } else {
                    egui::Color32::from_white_alpha(10)
                };

                painter.rect_filled(bar_rect, 3.0, bar_color);

                if val > 0.0 {
                    painter.text(
                        egui::pos2(bar_center_x, baseline_y - bar_height - 8.0),
                        egui::Align2::CENTER_CENTER,
                        format!("{}m", val as u32),
                        egui::FontId::proportional(8.5),
                        egui::Color32::from_white_alpha(180)
                    );
                }

                let date_day = dates[i];
                let label = match date_day.weekday() {
                    chrono::Weekday::Mon => "M",
                    chrono::Weekday::Tue => "T",
                    chrono::Weekday::Wed => "W",
                    chrono::Weekday::Thu => "T",
                    chrono::Weekday::Fri => "F",
                    chrono::Weekday::Sat => "S",
                    chrono::Weekday::Sun => "S",
                };

                painter.text(
                    egui::pos2(bar_center_x, baseline_y + 10.0),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(9.5),
                    egui::Color32::from_white_alpha(120)
                );
            }

            ui.add_space(10.0);

            // 3. Category distribution trackers
            let mut cat_totals: HashMap<String, u32> = HashMap::new();
            for entry in &self.history {
                if entry.mode == TimerMode::Work {
                    *cat_totals.entry(entry.category.clone()).or_default() += entry.duration_mins;
                }
            }

            let total_focus_mins: u32 = cat_totals.values().sum();
            if total_focus_mins > 0 {
                ui.label(egui::RichText::new("Time Allocation").strong().color(egui::Color32::WHITE));
                ui.add_space(4.0);

                // Sort by mins desc
                let mut cat_vec: Vec<(String, u32)> = cat_totals.into_iter().collect();
                cat_vec.sort_by(|a, b| b.1.cmp(&a.1));

                for (cat, mins) in cat_vec {
                    let progress = mins as f32 / total_focus_mins as f32;
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(cat).size(11.0).color(egui::Color32::from_white_alpha(180)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(format!("{} min ({:.0}%)", mins, progress * 100.0)).size(11.0).color(egui::Color32::from_white_alpha(120)));
                        });
                    });
                    ui.add(egui::ProgressBar::new(progress).show_percentage());
                    ui.add_space(6.0);
                }
            }
        });
    }

    fn render_history(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Workspace Logs").strong().color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑 Clear").clicked() {
                        self.show_clear_confirm = true;
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            if self.show_clear_confirm {
                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(255, 120, 120), "⚠ Clear all session history?");
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.add_space(30.0);
                            if ui.button("Yes, Clear").clicked() {
                                let _ = self.history_manager.clear_history();
                                self.history.clear();
                                self.show_clear_confirm = false;
                                self.persist_config();
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_clear_confirm = false;
                            }
                        });
                    });
                });
                ui.add_space(8.0);
            }

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Session Logs").strong().color(egui::Color32::WHITE));
            });
            ui.add_space(6.0);

            if self.history.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new("No focus sessions yet.").italics().color(egui::Color32::from_rgb(120, 120, 120)));
                });
            } else {
                let mut grouped: std::collections::BTreeMap<chrono::NaiveDate, Vec<HistoryEntry>> = std::collections::BTreeMap::new();
                for entry in &self.history {
                    let date = entry.timestamp.date_naive();
                    grouped.entry(date).or_default().push(entry.clone());
                }

                // Render scroll logs
                let log_height = if self.history.is_empty() { 80.0 } else { 150.0 };
                egui::ScrollArea::vertical().max_height(log_height).show(ui, |ui| {
                    for (date, entries) in grouped.iter().rev() {
                        ui.add_space(4.0);
                        let date_str = date.format("%b %d, %Y").to_string();
                        ui.label(egui::RichText::new(&date_str).strong().color(egui::Color32::from_rgb(140, 180, 255)));
                        ui.add_space(2.0);

                        for entry in entries.iter().rev() {
                            let title = format!(
                                "⏱ {} | {}m | [{}] {}",
                                entry.timestamp.format("%H:%M"),
                                entry.duration_mins,
                                entry.category,
                                entry.task_name
                            );

                            ui.collapsing(title, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(egui::RichText::new(&entry.notes).italics().color(egui::Color32::from_rgb(180, 180, 180)));
                                });
                            });
                        }
                    }
                });
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Render analytics bar chart and breakdowns in sidebar
            self.render_analytics(ui);
        });
    }

    fn render_timer_and_settings(&mut self, ui: &mut egui::Ui) {
        let is_editable = !self.is_running;
        ui.vertical(|ui| {
            // Header panel: title, status indicator, compact toggle
            ui.horizontal(|ui| {
                // Draw a beautiful glowing mini vector logo icon
                let logo_size = egui::vec2(28.0, 28.0);
                let (response, painter) = ui.allocate_painter(logo_size, egui::Sense::hover());
                let center = response.rect.center();
                let accent = self.get_theme_accent_color();
                
                // Ring backdrop
                painter.circle_stroke(center, 12.0, egui::Stroke::new(1.5, egui::Color32::from_white_alpha(40)));
                // Glowing accent ring
                painter.circle_stroke(center, 12.0, egui::Stroke::new(2.5, accent.linear_multiply(0.7)));
                
                // Center abstract audio sine wave instead of clashing bars
                let mut wave_points = Vec::new();
                for i in 0..12 {
                    let x = center.x - 7.0 + (i as f32 / 11.0) * 14.0;
                    let angle = (i as f32 / 11.0) * 2.0 * std::f32::consts::PI;
                    let y = center.y + angle.sin() * 4.0;
                    wave_points.push(egui::pos2(x, y));
                }
                for i in 0..11 {
                    painter.line_segment([wave_points[i], wave_points[i+1]], egui::Stroke::new(1.8, accent));
                }
                
                ui.add_space(6.0);
                ui.label(egui::RichText::new("Focus Flow").size(24.0).strong().color(egui::Color32::WHITE));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (status_text, bg, fg) = match self.status {
                        TimerStatus::Ready => ("READY", egui::Color32::from_rgb(40, 160, 80), egui::Color32::WHITE),
                        TimerStatus::Focusing => ("FOCUSING", egui::Color32::from_rgb(230, 90, 60), egui::Color32::WHITE),
                        TimerStatus::Paused => ("PAUSED", egui::Color32::from_rgb(200, 150, 40), egui::Color32::WHITE),
                        TimerStatus::Break => ("BREAK", egui::Color32::from_rgb(40, 160, 200), egui::Color32::WHITE),
                    };

                    Self::draw_badge(ui, status_text, bg, fg);

                    let streak = self.calculate_daily_streak();
                    if streak > 0 {
                        ui.add_space(8.0);
                        let streak_text = format!("🔥 {} Day Streak", streak);
                        Self::draw_badge(
                            ui,
                            &streak_text,
                            egui::Color32::from_rgb(255, 110, 80),
                            egui::Color32::BLACK
                        );
                    }

                    ui.add_space(8.0);
                    if ui.button("📺 Compact").clicked() {
                        self.compact_mode = true;
                    }
                    ui.add_space(8.0);
                    if ui.button("📥 Minimize to Tray").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Visible(false));
                    }
                });
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);

            // Category tags selection & Dynamic creation
            ui.label(egui::RichText::new("Focus Category").strong().color(egui::Color32::WHITE));
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                // Add tag inline
                ui.add_enabled_ui(is_editable, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_tag_input)
                            .hint_text("New tag name...")
                            .desired_width(120.0)
                    );
                    if ui.button("➕ Add").clicked() && !self.new_tag_input.trim().is_empty() {
                        let new_tag = self.new_tag_input.trim().to_string();
                        if !self.categories.contains(&new_tag) {
                            self.categories.push(new_tag);
                            self.persist_config();
                        }
                        self.new_tag_input.clear();
                    }
                });
            });

            ui.add_space(4.0);

            // Scrollable chips list
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    let categories = self.categories.clone();
                    for cat in &categories {
                        let is_selected = self.active_category == *cat;
                        let accent_color = self.get_theme_accent_color();

                        let chip_btn = if is_selected {
                            egui::Button::new(egui::RichText::new(cat).color(egui::Color32::BLACK))
                                .fill(accent_color)
                        } else {
                            egui::Button::new(egui::RichText::new(cat).color(egui::Color32::WHITE))
                                .fill(egui::Color32::from_white_alpha(15))
                        };

                        ui.add_enabled_ui(is_editable, |ui| {
                            if ui.add(chip_btn).clicked() {
                                self.active_category = cat.clone();
                                self.persist_config();
                            }
                        });
                    }
                });
            });

            ui.add_space(10.0);

            // Columns layout for Ambient Sound selection & Theme Preset selectors
            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    ui.label(egui::RichText::new("Background Sounds").strong().color(egui::Color32::WHITE));
                    ui.add_space(4.0);
                    let old_style = self.ambient_style;
                    egui::ComboBox::from_id_salt("ambient_combo")
                        .selected_text(match self.ambient_style {
                            AmbientStyle::None => "🔈 Mute / Silence",
                            AmbientStyle::SpacePad => "🎵 Warm Space Pad",
                            AmbientStyle::BinauralBeats => "🧠 Focus Binaural Beats",
                            AmbientStyle::WhiteNoise => "💨 White Noise Hiss",
                            AmbientStyle::BrownNoise => "🌊 Brown Noise Ocean",
                            AmbientStyle::LofiBeat => "☕ Cozy Lo-Fi Beats",
                            AmbientStyle::CozyRain => "🌧️ Cozy Rainfall (Procedural)",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.ambient_style, AmbientStyle::None, "🔈 Mute / Silence");
                            ui.selectable_value(&mut self.ambient_style, AmbientStyle::SpacePad, "🎵 Warm Space Pad");
                            ui.selectable_value(&mut self.ambient_style, AmbientStyle::BinauralBeats, "🧠 Focus Binaural Beats");
                            ui.selectable_value(&mut self.ambient_style, AmbientStyle::WhiteNoise, "💨 White Noise Hiss");
                            ui.selectable_value(&mut self.ambient_style, AmbientStyle::BrownNoise, "🌊 Brown Noise Ocean");
                            ui.selectable_value(&mut self.ambient_style, AmbientStyle::LofiBeat, "☕ Cozy Lo-Fi Beats");
                            ui.selectable_value(&mut self.ambient_style, AmbientStyle::CozyRain, "🌧️ Cozy Rainfall (Procedural)");
                        });
                    if self.ambient_style != old_style {
                        self.update_ambient_sound();
                        self.persist_config();
                    }
                });

                cols[1].vertical(|ui| {
                    ui.label(egui::RichText::new("Visual Workspace Theme").strong().color(egui::Color32::WHITE));
                    ui.add_space(4.0);
                    let old_theme = self.theme;
                    egui::ComboBox::from_id_salt("theme_preset_combo")
                        .selected_text(match self.theme {
                            ThemePreset::SunsetGlow => "🌅 Sunset Glow Theme",
                            ThemePreset::ForestWhisper => "🌿 Forest Whisper Theme",
                            ThemePreset::DeepCosmos => "🌌 Deep Cosmos Theme",
                            ThemePreset::Cyberpunk => "🌃 Cyberpunk Neon Theme",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.theme, ThemePreset::SunsetGlow, "🌅 Sunset Glow Theme");
                            ui.selectable_value(&mut self.theme, ThemePreset::ForestWhisper, "🌿 Forest Whisper Theme");
                            ui.selectable_value(&mut self.theme, ThemePreset::DeepCosmos, "🌌 Deep Cosmos Theme");
                            ui.selectable_value(&mut self.theme, ThemePreset::Cyberpunk, "🌃 Cyberpunk Neon Theme");
                        });
                    if self.theme != old_theme {
                        self.persist_config();
                    }
                });
            });

            ui.add_space(10.0);

            // Work/Break Duration Config Slider
            ui.add_enabled_ui(is_editable, |ui| {
                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        ui.label("Work Duration:");
                        let mut work_mins = self.work_duration_mins as f32;
                        if ui.add(egui::Slider::new(&mut work_mins, 1.0..=180.0).suffix(" min")).changed() {
                            self.work_duration_mins = work_mins.round() as u32;
                            if self.mode == TimerMode::Work && !self.is_running && self.status == TimerStatus::Ready {
                                self.time_remaining = self.work_duration_mins as f64 * 60.0;
                                self.total_duration = self.time_remaining;
                            }
                            self.persist_config();
                        }
                    });

                    cols[1].vertical(|ui| {
                        ui.label("Break Duration:");
                        let mut break_mins = self.break_duration_mins as f32;
                        if ui.add(egui::Slider::new(&mut break_mins, 1.0..=180.0).suffix(" min")).changed() {
                            self.break_duration_mins = break_mins.round() as u32;
                            if self.mode == TimerMode::Break && !self.is_running {
                                self.time_remaining = self.break_duration_mins as f64 * 60.0;
                                self.total_duration = self.time_remaining;
                            }
                            self.persist_config();
                        }
                    });
                });
            });

            ui.add_space(10.0);

            // Active Task Name & Todo Queue Panel enqueuer
            ui.add_enabled_ui(is_editable, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Task Title:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.task_name)
                            .hint_text("What are you focusing on right now?")
                            .desired_width(f32::INFINITY)
                    );
                });
            });

            ui.add_space(15.0);

            // Circular progress ring and timer text
            ui.vertical_centered(|ui| {
                let size = egui::vec2(220.0, 220.0);
                let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
                let center = response.rect.center();
                let radius = response.rect.width().min(response.rect.height()) * 0.45;

                // Inner circle backdrop shadow
                painter.circle_filled(center, radius - 4.0, egui::Color32::from_black_alpha(25));

                // Outer progress ring backdrop
                painter.circle_stroke(
                    center,
                    radius,
                    egui::Stroke::new(6.0, egui::Color32::from_white_alpha(15))
                );

                let progress_ratio = if self.total_duration > 0.0 {
                    (self.time_remaining / self.total_duration).clamp(0.0, 1.0) as f32
                } else {
                    0.0
                };

                if progress_ratio > 0.0 {
                    let mut points = Vec::new();
                    let num_segments = 120;
                    let start_angle = -std::f32::consts::FRAC_PI_2;
                    let total_angle = 2.0 * std::f32::consts::PI * progress_ratio;

                    for i in 0..=num_segments {
                        let angle = start_angle + (i as f32 / num_segments as f32) * total_angle;
                        let x = center.x + radius * angle.cos();
                        let y = center.y + radius * angle.sin();
                        points.push(egui::pos2(x, y));
                    }

                    let stroke_color = self.get_theme_accent_color();
                    painter.add(egui::Shape::line(points, egui::Stroke::new(10.0, stroke_color)));
                }

                // Render Timer text
                let mins = (self.time_remaining / 60.0).floor() as u32;
                let secs = (self.time_remaining % 60.0).floor() as u32;
                let time_str = format!("{:02}:{:02}", mins, secs);

                painter.text(
                    center - egui::vec2(0.0, 12.0),
                    egui::Align2::CENTER_CENTER,
                    time_str,
                    egui::FontId::proportional(48.0),
                    egui::Color32::WHITE
                );

                let label_str = match self.mode {
                    TimerMode::Work => "FOCUSING STATE",
                    TimerMode::Break => "BREATHE & REST",
                };

                let label_color = self.get_theme_accent_color();

                painter.text(
                    center + egui::vec2(0.0, 28.0),
                    egui::Align2::CENTER_CENTER,
                    label_str,
                    egui::FontId::proportional(11.0),
                    label_color
                );
            });

            ui.add_space(15.0);

            // Controls
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(16.0, 0.0);
                let button_width = (ui.available_width() - 32.0) / 3.0;

                // Start/Pause Button
                let play_pause_label = if self.is_running { "⏸ Pause" } else { "▶ Start" };
                let play_pause_btn = egui::Button::new(
                    egui::RichText::new(play_pause_label).size(15.0).strong()
                ).min_size(egui::vec2(button_width, 38.0));

                if ui.add(play_pause_btn).clicked() {
                    self.is_running = !self.is_running;
                    if self.is_running {
                        self.status = match self.mode {
                            TimerMode::Work => TimerStatus::Focusing,
                            TimerMode::Break => TimerStatus::Break,
                        };
                    } else {
                        self.status = TimerStatus::Paused;
                    }
                }

                // Skip Button
                let skip_btn = egui::Button::new(
                    egui::RichText::new("⏭ Skip").size(15.0)
                ).min_size(egui::vec2(button_width, 38.0));

                if ui.add(skip_btn).clicked() {
                    self.handle_skip();
                }

                // Reset Button
                let reset_btn = egui::Button::new(
                    egui::RichText::new("🔄 Reset").size(15.0)
                ).min_size(egui::vec2(button_width, 38.0));

                if ui.add(reset_btn).clicked() {
                    self.is_running = false;
                    self.mode = TimerMode::Work;
                    self.time_remaining = self.work_duration_mins as f64 * 60.0;
                    self.total_duration = self.time_remaining;
                    self.status = TimerStatus::Ready;
                    self.session_notes.clear();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Columns layout enqueuing TODO list (left side) and Session Notes (right side)
            ui.columns(2, |cols| {
                // Col 0: Todo List
                cols[0].vertical(|ui| {
                    ui.label(egui::RichText::new("Task Queue Checklist").strong().color(egui::Color32::WHITE));
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_todo_input)
                                .hint_text("Queue new task...")
                                .desired_width(120.0)
                        );
                        if ui.button("➕").clicked() && !self.new_todo_input.trim().is_empty() {
                            let item = TodoItem {
                                id: format!("{}", chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)),
                                title: self.new_todo_input.trim().to_string(),
                                completed: false,
                            };
                            self.task_queue.push(item);
                            self.new_todo_input.clear();
                            self.persist_config();
                        }
                    });

                    ui.add_space(4.0);

                    // Render Checklist
                    let scroll_height = if self.task_queue.is_empty() { 40.0 } else { 100.0 };
                    egui::ScrollArea::vertical().max_height(scroll_height).show(ui, |ui| {
                        let mut toggle_dirty = false;
                        for item in &mut self.task_queue {
                            ui.horizontal(|ui| {
                                let mut comp = item.completed;
                                if ui.checkbox(&mut comp, "").changed() {
                                    item.completed = comp;
                                    toggle_dirty = true;
                                }

                                // Clicking the task title loads it into the active timer name
                                let label_color = if item.completed {
                                    egui::Color32::from_white_alpha(100)
                                } else {
                                    egui::Color32::WHITE
                                };

                                let title_text = if item.completed {
                                    egui::RichText::new(&item.title).strikethrough().color(label_color)
                                } else {
                                    egui::RichText::new(&item.title).color(label_color)
                                };

                                if ui.selectable_label(self.task_name == item.title, title_text).clicked() && !item.completed {
                                    if is_editable {
                                        self.task_name = item.title.clone();
                                    }
                                }
                            });
                        }
                        if toggle_dirty {
                            self.persist_config();
                        }
                    });
                });

                // Col 1: Session Notes (Work) / Breathing sphere (Break)
                cols[1].vertical(|ui| {
                    match self.mode {
                        TimerMode::Work => {
                            ui.label(egui::RichText::new("Session Notes").strong().color(egui::Color32::WHITE));
                            ui.add_space(4.0);
                            ui.add(
                                egui::TextEdit::multiline(&mut self.session_notes)
                                    .hint_text("Jot notes during this work session...")
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(4)
                            );
                        }
                        TimerMode::Break => {
                            let breathing_val = self.get_breathing_value();
                            let text = self.get_breathing_text();

                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("BREATHING SPHERE").strong().size(11.0).color(egui::Color32::from_rgb(140, 200, 255)));
                                ui.add_space(4.0);

                                let (response, painter) = ui.allocate_painter(egui::vec2(180.0, 75.0), egui::Sense::hover());
                                let center = response.rect.center();

                                let min_r = 12.0;
                                let max_r = 30.0;
                                let r = min_r + (max_r - min_r) * breathing_val;

                                let glow_color = egui::Color32::from_rgba_unmultiplied(
                                    80, 220, 255,
                                    (25.0 + 35.0 * breathing_val) as u8
                                );
                                painter.circle_filled(center, r + 4.0, glow_color);

                                let sphere_color = self.get_theme_accent_color();
                                painter.circle_filled(center, r, sphere_color);

                                ui.add_space(4.0);
                                ui.label(egui::RichText::new(text).size(13.0).strong().color(egui::Color32::WHITE));
                            });
                        }
                    }
                });
            });
        });
    }

    fn render_compact_mode(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // Compact progress circle
                let size = egui::vec2(70.0, 70.0);
                let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
                let center = response.rect.center();
                let radius = response.rect.width().min(response.rect.height()) * 0.44;

                // Outer progress ring backdrop
                painter.circle_stroke(
                    center,
                    radius,
                    egui::Stroke::new(3.5, egui::Color32::from_white_alpha(15))
                );

                let progress_ratio = if self.total_duration > 0.0 {
                    (self.time_remaining / self.total_duration).clamp(0.0, 1.0) as f32
                } else {
                    0.0
                };

                if progress_ratio > 0.0 {
                    let mut points = Vec::new();
                    let num_segments = 64;
                    let start_angle = -std::f32::consts::FRAC_PI_2;
                    let total_angle = 2.0 * std::f32::consts::PI * progress_ratio;

                    for i in 0..=num_segments {
                        let angle = start_angle + (i as f32 / num_segments as f32) * total_angle;
                        let x = center.x + radius * angle.cos();
                        let y = center.y + radius * angle.sin();
                        points.push(egui::pos2(x, y));
                    }

                    let stroke_color = self.get_theme_accent_color();
                    painter.add(egui::Shape::line(points, egui::Stroke::new(5.0, stroke_color)));
                }

                // Centered timer text
                let mins = (self.time_remaining / 60.0).floor() as u32;
                let secs = (self.time_remaining % 60.0).floor() as u32;
                let time_str = format!("{:02}:{:02}", mins, secs);

                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    time_str,
                    egui::FontId::proportional(13.0),
                    egui::Color32::WHITE
                );

                // Inline Controls and Compact Exit Buttons
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let play_pause_label = if self.is_running { "⏸" } else { "▶" };
                        let accent = self.get_theme_accent_color();
                        let btn = egui::Button::new(egui::RichText::new(play_pause_label).strong().color(accent));
                        if ui.add(btn).clicked() {
                            self.is_running = !self.is_running;
                            self.status = if self.is_running {
                                match self.mode {
                                    TimerMode::Work => TimerStatus::Focusing,
                                    TimerMode::Break => TimerStatus::Break,
                                }
                            } else {
                                TimerStatus::Paused
                            };
                        }

                        if ui.button("⏭").clicked() {
                            self.handle_skip();
                        }
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        if ui.button("📺 Full").clicked() {
                            self.compact_mode = false;
                        }
                        if ui.button("📥 Tray").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Visible(false));
                        }
                    });
                });
            });
        });
    }
}

impl eframe::App for FocusFlowApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Cleanup completed dynamic audio sinks to prevent memory/resource leaks
        self.dynamic_sinks.retain(|sink| !sink.empty());

        let ctx = ui.ctx();
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64();
        self.last_tick = now;

        // Cap dt to handle pauses or background suspension gracefully
        let dt = dt.min(0.1);

        if self.is_running {
            self.time_remaining -= dt;
            if self.mode == TimerMode::Work {
                // Hue shifts continuously by 3 degrees per second during focus
                self.hue_accumulator = (self.hue_accumulator + dt as f32 * 3.0) % 360.0;
            }
            if self.time_remaining <= 0.0 {
                self.time_remaining = 0.0;
                self.is_running = false;
                self.handle_timer_complete();
            }
        }

        if self.mode == TimerMode::Break {
            self.breathing_accumulator = (self.breathing_accumulator + dt as f32) % 16.0;
        }

        // Procedural music/ambient noise state synthesizer clocks
        if self.ambient_style == AmbientStyle::SpacePad {
            self.ambient_timer += dt as f32;
            if self.ambient_timer >= 5.0 {
                let sinks = self.audio_player.play_ambient_chord(self.ambient_chord_index);
                self.dynamic_sinks.extend(sinks);
                self.ambient_chord_index = (self.ambient_chord_index + 1) % 1000;
                self.ambient_timer = 0.0;
            }
        } else if self.ambient_style == AmbientStyle::LofiBeat {
            self.ambient_timer += dt as f32;
            // 75 BPM Boom-Bap Sequencer Beat tick clock
            if self.ambient_timer >= 0.20 {
                let current_step = self.ambient_chord_index;
                self.ambient_chord_index = (self.ambient_chord_index + 1) % 64;
                self.ambient_timer -= 0.20; // Maintain absolute rhythmic grid precision

                let step_in_bar = current_step % 16;
                let bar = current_step / 16;

                if let Some(ref handle) = self.audio_player.stream_handle {
                    // 1. Trigger Rhodes Jazz Chords (Dm9 -> G13 -> Cmaj9 -> Am9)
                    if step_in_bar == 0 {
                        let chords = [
                            vec![146.83, 174.61, 220.00, 261.63, 329.63], // Dm9: D3, F3, A3, C4, E4
                            vec![98.00, 246.94, 293.66, 349.23, 440.00],  // G13: G2, B3, D4, F4, A4
                            vec![130.81, 164.81, 196.00, 246.94, 293.66], // Cmaj9: C3, E3, G3, B3, D4
                            vec![110.00, 130.81, 164.81, 196.00, 246.94], // Am9: A2, C3, E3, G3, B3
                        ];
                        let freqs = &chords[bar % chords.len()];
                        for &freq in freqs {
                            if let Ok(sink) = rodio::Sink::try_new(handle) {
                                let source = SineWave::new(freq)
                                    .take_duration(std::time::Duration::from_secs(3))
                                    .fade_in(std::time::Duration::from_millis(150))
                                    .fade_out(std::time::Duration::from_secs(2))
                                    .amplify(0.035); // Cozy mellow Rhodes sound (amplified)
                                sink.append(source);
                                self.dynamic_sinks.push(sink);
                            }
                        }
                    }

                    // 2. Boom-Bap Kick Sweep: Steps 0, 6, 8, 11
                    if step_in_bar == 0 || step_in_bar == 6 || step_in_bar == 8 || step_in_bar == 11 {
                        if let Ok(sink) = rodio::Sink::try_new(handle) {
                            let source = SineWave::new(55.0)
                                .take_duration(std::time::Duration::from_millis(85))
                                .fade_out(std::time::Duration::from_millis(40))
                                .amplify(0.25); // Kick thud (amplified)
                            sink.append(source);
                            self.dynamic_sinks.push(sink);
                        }
                    }

                    // 3. Boom-Bap Snare: Steps 4, 12
                    if step_in_bar == 4 || step_in_bar == 12 {
                        if let Ok(sink) = rodio::Sink::try_new(handle) {
                            let source = NoiseSource::new(NoiseStyle::White)
                                .take_duration(std::time::Duration::from_millis(100))
                                .fade_out(std::time::Duration::from_millis(70))
                                .amplify(0.035); // Snare snap (amplified)
                            sink.append(source);
                            self.dynamic_sinks.push(sink);
                        }
                    }

                    // 4. Boom-Bap Hi-Hat: Steps 2, 6, 10, 14
                    if step_in_bar == 2 || step_in_bar == 6 || step_in_bar == 10 || step_in_bar == 14 {
                        if let Ok(sink) = rodio::Sink::try_new(handle) {
                            let source = NoiseSource::new(NoiseStyle::White)
                                .take_duration(std::time::Duration::from_millis(30))
                                .fade_out(std::time::Duration::from_millis(15))
                                .amplify(0.018); // Hi-Hat click (amplified)
                            sink.append(source);
                            self.dynamic_sinks.push(sink);
                        }
                    }

                    // 5. Random Cozy Vinyl Crackle clicks (5% chance per step)
                    let rand_val = (self.hue_accumulator * 12345.67).sin().fract().abs();
                    if rand_val > 0.94 {
                        if let Ok(sink) = rodio::Sink::try_new(handle) {
                            let source = NoiseSource::new(NoiseStyle::White)
                                .take_duration(std::time::Duration::from_millis(5))
                                .amplify(0.025); // Dusty pop click (amplified)
                            sink.append(source);
                            self.dynamic_sinks.push(sink);
                        }
                    }
                }
            }
        }

        // Repaint at 60 FPS for ultra-smooth HSL color cycling and breathing sphere pulses
        ctx.request_repaint_after(std::time::Duration::from_millis(16));

        // Background calculation
        let bg_color = self.get_theme_bg_color();

        // Compact Mode dynamic window resizing & Always on Top configurations
        if self.compact_mode {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(240.0, 100.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(240.0, 100.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
        } else {
            if self.last_compact_state {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(900.0, 720.0)));
                ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(720.0, 580.0)));
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
            }
        }
        self.last_compact_state = self.compact_mode;

        // Custom container styles
        let central_frame = egui::Frame::NONE.fill(bg_color).inner_margin(if self.compact_mode { 8.0 } else { 24.0 });
        let side_frame = egui::Frame::NONE
            .fill(bg_color.linear_multiply(0.85)) // Blend darker for history analytics panel
            .inner_margin(20.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(10)));

        if self.compact_mode {
            egui::CentralPanel::default().frame(central_frame).show_inside(ui, |ui| {
                self.render_compact_mode(ui);
            });
        } else {
            egui::SidePanel::right("history_panel")
                .frame(side_frame)
                .resizable(true)
                .default_size(320.0)
                .min_size(260.0)
                .show_inside(ui, |ui| {
                    self.render_history(ui);
                });

            egui::CentralPanel::default().frame(central_frame).show_inside(ui, |ui| {
                self.render_timer_and_settings(ui);
            });
        }
    }
}

fn load_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../logo.png");
    if let Ok(image) = image::load_from_memory(icon_bytes) {
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();
        Some(egui::IconData {
            rgba: image.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([900.0, 720.0])
        .with_min_inner_size([720.0, 580.0])
        .with_title("Focus Flow");

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Focus Flow",
        options,
        Box::new(|cc| Ok(Box::new(FocusFlowApp::new(cc)))),
    )
}
