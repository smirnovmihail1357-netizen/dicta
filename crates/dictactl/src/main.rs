use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use clap::{Parser, Subcommand};
use dicta_core::{capture, load_wav, resample_to_16k, Config, Engine};

#[derive(Parser)]
#[command(
    name = "dictactl",
    about = "dicta: системный голосовой ввод (диагностика и управление)"
)]
struct Cli {
    /// Путь к конфигу (по умолчанию ~/.config/dicta/config.toml)
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Переопределить PipeWire-источник (pactl list short sources)
    #[arg(long, global = true)]
    mic: Option<String>,

    /// В какое окно вставлять (id приложения: konsole, firefox, kate...).
    /// Адаптеры горячей клавиши получают это из композитора.
    #[arg(long, global = true)]
    app: Option<String>,

    /// Не использовать GPU (CPU-бэкенд, не трогает VRAM)
    #[arg(long, global = true)]
    cpu: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Распознать WAV-файл
    Infer {
        wav: PathBuf,
        #[arg(long)]
        timing: bool,
    },
    /// Записать N секунд с микрофона в WAV
    Record {
        out: PathBuf,
        #[arg(long, default_value_t = 5)]
        secs: u64,
    },
    /// Записать N секунд и сразу распознать
    Dictate {
        #[arg(long, default_value_t = 5)]
        secs: u64,
        #[arg(long)]
        timing: bool,
    },
    /// Начать запись (демон)
    Start,
    /// Закончить запись, распознать и вставить в фокусное окно (демон)
    Stop,
    /// Alt+V: начать или закончить (демон)
    Toggle,
    /// Отменить запись без распознавания (Esc)
    Cancel,
    /// Состояние демона
    Status,
    /// Проверить, что uinput-клавиши доходят до приложений
    TestKeys { combo: String },
    /// Какое окно демон считает фокусным и какие клавиши вставки он выберет
    PasteKeys,
    /// Дерево доступности: какие приложения отдают AT-SPI и где у них фокус
    AtspiDump {
        #[arg(long, default_value_t = 8)]
        depth: usize,
        /// Печатать само дерево объектов (шумно; видно, где обход упирается в лимит)
        #[arg(long)]
        tree: bool,
    },
    /// Проверка AT-SPI-вставки напрямую, без демона: пишет в фокусный объект окна
    AtspiInsert { text: String },
    /// Вставить текст в активное окно (проверка вставки без записи и распознавания)
    Inject {
        text: String,
        /// Принудительные клавиши вставки, напр. shift+insert (по умолчанию — по окну)
        #[arg(long)]
        keys: Option<String>,
    },
    /// Записать N секунд и показать измеренные уровни (чтобы подобрать min_level)
    Calibrate {
        #[arg(long, default_value_t = 8)]
        secs: u64,
    },
    /// Сообщить демону-обёртке, какое окно сейчас фокусное (для ~/.cache/dicta/focus)
    FocusReport { app: String },
}

fn ask_json(socket: &std::path::Path, payload: &str) -> Result<String> {
    use std::io::{BufRead, BufReader, Write};
    let stream = std::os::unix::net::UnixStream::connect(socket).map_err(|err| {
        anyhow::anyhow!(
            "демон не отвечает на {} ({err}) — запусти dictad",
            socket.display()
        )
    })?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(120)))?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    writer.write_all(payload.as_bytes())?;
    writer.flush()?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn ask(socket: &std::path::Path, cmd: &str, app: Option<&str>) -> Result<String> {
    use std::io::{BufRead, BufReader, Write};
    let stream = std::os::unix::net::UnixStream::connect(socket).map_err(|err| {
        anyhow::anyhow!(
            "демон не отвечает на {} ({err}) — запусти dictad",
            socket.display()
        )
    })?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(120)))?;
    let payload = match app {
        Some(app) => {
            let app = app.replace(['"', '\\'], "");
            format!("{{\"cmd\":\"{cmd}\",\"app\":\"{app}\"}}\n")
        }
        None => format!("{{\"cmd\":\"{cmd}\"}}\n"),
    };
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    writer.write_all(payload.as_bytes())?;
    writer.flush()?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut cfg = Config::load(cli.config.as_deref())?;
    if let Some(mic) = cli.mic {
        cfg.mic = Some(mic);
    }
    if cli.cpu {
        cfg.use_gpu = false;
    }

    match cli.command {
        Command::Infer { wav, timing } => {
            let t0 = Instant::now();
            let engine = Engine::load(&cfg)?;
            let load = t0.elapsed();

            let t1 = Instant::now();
            let (samples, rate) = load_wav(&wav)?;
            let samples = resample_to_16k(samples, rate);
            let text = engine.transcribe(&samples)?;
            let infer = t1.elapsed();

            println!("{}", text);
            if timing {
                eprintln!(
                    "model={} lang={} multilingual={} gpu={} load={:?} infer={:?} samples={}",
                    cfg.model.display(),
                    cfg.language,
                    engine.is_multilingual(),
                    cfg.use_gpu,
                    load,
                    infer,
                    samples.len()
                );
            }
        }
        Command::Record { out, secs } => {
            capture::record_to(&cfg, secs, &out)?;
            println!("записано {} с -> {}", secs, out.display());
        }
        Command::Dictate { secs, timing } => {
            // Сначала запись, модель прогревается пока говоришь.
            let rec = capture::Recorder::start(&cfg)?;
            eprintln!("говорите {} с…", secs);

            let t0 = Instant::now();
            let engine = Engine::load(&cfg)?;
            let load = t0.elapsed();

            std::thread::sleep(std::time::Duration::from_secs(secs));
            let samples = rec.stop()?;

            let t1 = Instant::now();
            let text = engine.transcribe(&samples)?;
            let infer = t1.elapsed();

            println!("{}", text);
            if timing {
                eprintln!(
                    "load={:?} infer={:?} samples={}",
                    load,
                    infer,
                    samples.len()
                );
            }
        }
        Command::TestKeys { combo } => {
            dicta_core::inject::test_keys(&combo)?;
        }
        Command::Start => println!("{}", ask(&cfg.socket, "start", cli.app.as_deref())?),
        Command::Stop => println!("{}", ask(&cfg.socket, "stop", cli.app.as_deref())?),
        Command::Toggle => println!("{}", ask(&cfg.socket, "toggle", cli.app.as_deref())?),
        Command::Cancel => println!("{}", ask(&cfg.socket, "cancel", cli.app.as_deref())?),
        Command::Status => println!("{}", ask(&cfg.socket, "status", None)?),
        Command::AtspiDump { depth, tree } => {
            println!(
                "{}",
                dicta_core::atspi::dump(cli.app.as_deref(), depth, tree)?
            );
        }
        Command::AtspiInsert { text } => {
            let text = text.replace(['"', '\\'], "");
            let dst = dicta_core::atspi::insert_text_with(
                &text,
                cli.app.as_deref(),
                cfg.atspi_allow_any_role,
            )?;
            println!("вставлено в {dst}");
        }
        Command::Inject { text, keys } => {
            let text = text.replace(['"', '\\'], "");
            let app = cli
                .app
                .as_deref()
                .map(|a| a.replace(['"', '\\'], ""))
                .map(|a| format!(",\"app\":\"{a}\""))
                .unwrap_or_default();
            let keys_opt = keys
                .as_deref()
                .map(|k| format!(",\"keys\":\"{}\"", k.replace(['"', '\\'], "")))
                .unwrap_or_default();
            println!(
                "{}",
                ask_json(
                    &cfg.socket,
                    &format!("{{\"cmd\":\"inject\",\"text\":\"{text}\"{app}{keys_opt}}}\n")
                )?
            );
        }
        Command::Calibrate { secs } => {
            println!("говори все {secs} с (считай это тренировкой диктовки), потом ~2 с помолчи");
            let samples = capture::record_samples(&cfg, secs)?;
            let peak = dicta_core::speech_level(&samples);
            let mean = dicta_core::normalized_rms(&samples);
            let head = &samples[..(samples.len().min(16000 * 2))];
            println!(
                "пик речи={peak:.3}  в среднем={mean:.3}  первые 2 с={:.3}\n\
                 min_level в конфиге ставят между «фоном» и «речью»; сейчас по умолчанию 0.35",
                dicta_core::speech_level(head)
            );
        }
        Command::FocusReport { app } => {
            dicta_core::focus::write_focus_cache(&app)?;
            println!("{}", dicta_core::focus::focus_cache_path().display());
        }
        Command::PasteKeys => {
            let app = dicta_core::focus::active_app(cli.app.as_deref());
            let keys = cfg.paste_keys_for(app.as_deref());
            println!(
                "окно: {}\nклавиши: {} (по умолчанию: {})",
                app.unwrap_or_else(|| "не определено".into()),
                keys,
                cfg.paste_keys
            );
        }
    }

    Ok(())
}
