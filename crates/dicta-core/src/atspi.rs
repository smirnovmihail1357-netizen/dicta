//! Вставка текста через AT-SPI2 (D-Bus accessibility), без буфера обмена и без
//! синтезированных клавиш.
//!
//! Это альтернативный путь вставки (`inject_mode = "atspi"`, сборка с `--features atspi`):
//! демон находит в дереве доступности объект со состоянием `focused` и пишет текст
//! через интерфейс `org.a11y.atspi.EditableText`.
//!
//! Честные рамки метода:
//!   * работает там, где приложение отдаёт AT-SPI и `EditableText`
//!     (Firefox — да, GTK — да; Qt — только с `QT_ACCESSIBILITY=1`;
//!     Chromium/Electron — только с `--force-renderer-accessibility`);
//!   * дерево обходится breadth-first с ограничением по глубине и числу объектов
//!     (Firefox огромен: один кадр страницы — десятки детей);
//!   * пишем **только** в фокусный объект: случайное поле затронуто не будет.
//!
//! Как правильно искать приложения (главный вывод, на котором сначала споткнулись):
//! приложения в a11y-бусе зарегистрированы **уникальными** именами соединений
//! (`:1.13` = Firefox), а `org.a11y.atspi.Registry` держит их корневые объекты как
//! детей. Поэтому `DBusProxy::list_names()` (он отдаёт только well-known имена)
//! показывал «в бусе один реестр» — это был артефакт способа перечисления, а не
//! состояние системы. Проверяется одной командой:
//!
//! ```text
//! busctl --address=unix:path=/run/user/1000/at-spi/bus_0 \
//!   call org.a11y.atspi.Registry /org/a11y/atspi/accessible/root \
//!   org.a11y.atspi.Accessible GetChildren
//! ```
//!
//! Почему обход идёт уровнями, а не одним объектом за раз: последовательный обход
//! 100-200 объектов Firefox — это сотни D-Bus round trip'ов, то есть секунды и
//! десятки (замерено: 12 с и уход в дедлайн). Параллельно по `CONCURRENCY` запросов
//! тот же обход укладывается в сотни миллисекунд.
//!
//! `org.a11y.atspi.Collection.GetMatches` (запрос «найди объект со свойством X» одним
//! вызовом) здесь сознательно **не** используется: Firefox на этот запрос отвечает
//! пустым списком даже для `States=focusable` (проверено busctl'ом), то есть путь
//! непредсказуем. Надёжен только обход дерева.
//!
//! Диагностика: `dictactl atspi-dump` показывает приложения, роли, фокусные объекты
//! и тайминги этапов, так что поддержку конкретного окна можно проверить, ничего не
//! вставляя.

use anyhow::Result;

#[cfg(feature = "atspi")]
use std::time::Duration;

#[cfg(feature = "atspi")]
mod imp {
    use anyhow::{bail, Context, Result};
    use atspi::proxy::accessible::AccessibleProxy;
    use atspi::proxy::editable_text::EditableTextProxy;
    use atspi::proxy::text::TextProxy;
    use atspi::{Interface, Role, State};
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Instant;
    use zbus::connection::Builder as ConnectionBuilder;
    use zbus::Connection;

    const ROOT: &str = "/org/a11y/atspi/accessible/root";
    const REGISTRY: &str = "org.a11y.atspi.Registry";

    /// Потолок объектов на одно приложение: дерево Firefox заканчивать нечем.
    const MAX_OBJECTS: usize = 400;
    /// Сколько D-Bus-вызовов держим одновременно при обходе уровня.
    const CONCURRENCY: usize = 12;
    /// Глубина обхода для вставки (Firefox: app → frame → document → секции → поле).
    const INSERT_DEPTH: usize = 12;

    /// Замер длительностей этапов — чтобы «12 с» было видно по частям, а не одним фактом.
    struct Timer {
        last: Instant,
        stages: Vec<(String, u128)>,
    }

    impl Timer {
        fn new() -> Self {
            Self {
                last: Instant::now(),
                stages: Vec::new(),
            }
        }

        /// Записать, сколько прошло с предыдущей отметки.
        fn mark(&mut self, name: impl Into<String>) {
            let now = Instant::now();
            self.stages
                .push((name.into(), now.duration_since(self.last).as_millis()));
            self.last = now;
        }

        fn line(&self) -> String {
            self.stages
                .iter()
                .map(|(name, ms)| format!("{name} {ms} мс"))
                .collect::<Vec<_>>()
                .join(" | ")
        }

        fn total_ms(&self) -> u128 {
            self.stages.iter().map(|(_, ms)| ms).sum()
        }
    }

    /// Соединение с a11y-бусом (его адрес отдаёт session bus через `org.a11y.Bus`)
    /// плюс счётчик сделанных D-Bus-вызовов.
    struct Bus {
        conn: Connection,
        calls: AtomicU32,
    }

    impl Bus {
        fn counted(&self) {
            self.calls.fetch_add(1, Ordering::Relaxed);
        }

        async fn accessible<'a>(
            &'a self,
            dest: &'a str,
            path: &'a str,
        ) -> Result<AccessibleProxy<'a>> {
            self.counted();
            AccessibleProxy::builder(&self.conn)
                .destination(dest)?
                .path(path)?
                .build()
                .await
                .with_context(|| format!("нет accessible {dest} {path}"))
        }

        async fn connect(timer: &mut Timer) -> Result<Bus> {
            let session = Connection::session().await.context("нет session bus")?;
            timer.mark("session bus");
            let reply = session
                .call_method(
                    Some("org.a11y.Bus"),
                    "/org/a11y/bus",
                    Some("org.a11y.Bus"),
                    "GetAddress",
                    &(),
                )
                .await
                .context("org.a11y.Bus не отвечает (at-spi2-registryd не запущен?)")?;
            timer.mark("GetAddress");
            let addr: String = reply
                .body()
                .deserialize()
                .context("не прочитал адрес a11y-буса")?;
            let conn = ConnectionBuilder::address(addr.as_str())
                .context("некорректный адрес a11y-буса")?
                .build()
                .await
                .context("не подключился к a11y-бусу")?;
            timer.mark("a11y bus");
            Ok(Bus {
                conn,
                calls: AtomicU32::new(0),
            })
        }

        /// Приложения, видимые в дереве доступности: (имя соединения, имя приложения).
        /// Их перечисляет корневой объект реестра; адресуются они уникальными именами.
        async fn apps(&self) -> Result<Vec<(String, String)>> {
            let registry = self
                .accessible(REGISTRY, ROOT)
                .await
                .context("нет корневого объекта реестра AT-SPI")?;
            let kids = registry
                .get_children()
                .await
                .context("реестр AT-SPI не вернул список приложений")?;
            let mut out = Vec::new();
            for child in kids {
                let dest = child.name.to_string();
                let path = child.path.to_string();
                let Ok(obj) = self.accessible(&dest, &path).await else {
                    continue;
                };
                let app_name = obj.name().await.unwrap_or_default();
                if app_name.is_empty() {
                    continue;
                }
                out.push((dest, app_name));
            }
            Ok(out)
        }
    }

    /// Роль, которая означает «сюда пишут текст».
    ///
    /// Нужен именно фильтр по роли: фокусным бывает не только поле ввода. Firefox,
    /// например, отдаёт состояние `focused` самому узлу `document web` (и тот честно
    /// объявляет EditableText). Писать в документ вместо поля — уже не «вставка в
    /// фокусное поле», поэтому такие узлы пропускаем, кроме явного разрешения.
    fn looks_like_input_field(role: Role) -> bool {
        matches!(
            role,
            Role::Entry | Role::PasswordText | Role::ComboBox | Role::Text
        )
    }

    #[derive(Debug)]
    struct Found {
        dest: String,
        path: String,
        role: Role,
        name: String,
        /// Объект сам сообщает, что реализует org.a11y.atspi.EditableText.
        editable: bool,
    }

    /// Обход дерева одного приложения (по уровням, параллельно) в поисках фокусных объектов.
    async fn find_focused(
        bus: &Bus,
        dest: &str,
        max_depth: usize,
        verbose: bool,
    ) -> Result<(Vec<Found>, usize)> {
        let mut seen: HashSet<(String, String)> = HashSet::new();
        seen.insert((dest.to_string(), ROOT.to_string()));
        let mut level = vec![(dest.to_string(), ROOT.to_string())];
        let mut focused: Vec<Found> = Vec::new();
        let mut visited = 0usize;

        for depth in 0..max_depth {
            if level.is_empty() || seen.len() >= MAX_OBJECTS {
                break;
            }
            let mut next: Vec<(String, String)> = Vec::new();
            for chunk in level.chunks(CONCURRENCY) {
                let results = futures::future::join_all(chunk.iter().map(|(d, p)| async move {
                    let Ok(obj) = bus.accessible(d, p).await else {
                        return None;
                    };
                    let state = obj.get_state().await.unwrap_or_default();
                    let kids = obj.get_children().await.unwrap_or_default();
                    let is_focused = state.contains(State::Focused);
                    // Роль, имя и интерфейсы нужны для дампа и для описания фокуса.
                    let describe = is_focused || verbose;
                    let role = if describe {
                        obj.get_role().await.unwrap_or(Role::Unknown)
                    } else {
                        Role::Unknown
                    };
                    let name = if describe {
                        obj.name().await.unwrap_or_default()
                    } else {
                        String::new()
                    };
                    // editable берём из самого объекта: построить прокси локально — не
                    // значит, что приложение интерфейс отдаёт.
                    let editable = if is_focused {
                        obj.get_interfaces()
                            .await
                            .map(|i| i.contains(Interface::EditableText))
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    let children: Vec<(String, String)> = kids
                        .iter()
                        .map(|k| (k.name.to_string(), k.path.to_string()))
                        .collect();
                    Some((
                        is_focused,
                        role,
                        name,
                        editable,
                        children,
                        d.clone(),
                        p.clone(),
                    ))
                }))
                .await;

                for item in results.into_iter().flatten() {
                    let (is_focused, role, name, editable, children, d, p) = item;
                    visited += 1;
                    if verbose {
                        println!(
                            "{}{role}  name={name:?} children={} focused={is_focused} editable={editable}",
                            " ".repeat(depth * 2),
                            children.len()
                        );
                    }
                    if is_focused {
                        focused.push(Found {
                            dest: d.clone(),
                            path: p.clone(),
                            role,
                            name,
                            editable,
                        });
                    }
                    if depth + 1 >= max_depth || seen.len() >= MAX_OBJECTS {
                        continue;
                    }
                    for (cd, cp) in children {
                        let key = (cd.clone(), cp.clone());
                        if !seen.contains(&key) {
                            seen.insert(key);
                            next.push((cd, cp));
                        }
                    }
                }
            }
            level = next;
        }
        Ok((focused, visited))
    }

    /// Вставь текст в фокусный редактируемый объект (перебирая приложения).
    /// Возвращает «куда именно» (bus name + путь), чтобы это было видно в логе.
    pub async fn insert(
        text: String,
        app_hint: Option<String>,
        allow_any_role: bool,
    ) -> Result<String> {
        let mut timer = Timer::new();
        let bus = Bus::connect(&mut timer).await?;
        let apps = bus.apps().await?;
        timer.mark("приложения");

        // Подсказка окна — ограничение, а не условие: если по имени окна в дереве
        // ничего нет (DE называет окно иначе, чем AT-SPI), ищем по всем приложениям.
        let hint = app_hint.map(|h| h.to_lowercase());
        let candidates: Vec<(String, String)> = match &hint {
            Some(h) => {
                let picked: Vec<(String, String)> = apps
                    .iter()
                    .filter(|(_, name)| name.to_lowercase().contains(h.as_str()))
                    .cloned()
                    .collect();
                if picked.is_empty() {
                    eprintln!(
                        "dicta: atspi: окно {h:?} не найдено в дереве доступности, ищем по всем"
                    );
                    apps
                } else {
                    picked
                }
            }
            None => apps,
        };
        let mut tried: Vec<String> = Vec::new();
        let mut visited = 0usize;

        for (dest, app_name) in candidates {
            let (focused, walked) = find_focused(&bus, &dest, INSERT_DEPTH, false).await?;
            visited += walked;
            timer.mark(format!("обход {app_name}"));

            for found in focused {
                if !found.editable {
                    tried.push(format!("{} ({}, EditableText нет)", found.path, found.role));
                    continue;
                }
                if !allow_any_role && !looks_like_input_field(found.role) {
                    tried.push(format!(
                        "{} (роль {} — не поле ввода)",
                        found.path, found.role
                    ));
                    continue;
                }
                let Ok(editable) = EditableTextProxy::builder(&bus.conn)
                    .destination(found.dest.as_str())?
                    .path(found.path.as_str())?
                    .build()
                    .await
                else {
                    tried.push(format!(
                        "{} ({}) нет прокси EditableText",
                        found.path, found.role
                    ));
                    continue;
                };
                let caret = match TextProxy::builder(&bus.conn)
                    .destination(found.dest.as_str())?
                    .path(found.path.as_str())?
                    .build()
                    .await
                {
                    Ok(t) => t.caret_offset().await.unwrap_or(-1),
                    Err(_) => -1,
                };
                let len = text.chars().count() as i32;
                let res = if caret >= 0 {
                    editable.insert_text(caret, &text, len).await
                } else {
                    editable.set_text_contents(&text).await
                };
                match res {
                    Ok(true) => {
                        timer.mark("запись");
                        if timer.total_ms() > 800 {
                            eprintln!(
                                "dicta: atspi этапы: {} (объектов пройдено: {visited})",
                                timer.line()
                            );
                        }
                        return Ok(format!(
                            "{} {} роль={} name={:?}",
                            found.dest, found.path, found.role, found.name
                        ));
                    }
                    Ok(false) => tried.push(format!("{} ({})", found.path, found.role)),
                    Err(err) => tried.push(format!("{} ({}): {err}", found.path, found.role)),
                }
            }
        }

        timer.mark("всего");
        eprintln!(
            "dicta: atspi этапы: {} (объектов пройдено: {visited}, D-Bus-вызовов: {})",
            timer.line(),
            bus.calls.load(Ordering::Relaxed)
        );
        bail!(
            "AT-SPI: фокусный редактируемый объект не найден ({}); \
             проверь поддержку командой dictactl atspi-dump",
            if tried.is_empty() {
                "нет focused-объектов".to_string()
            } else {
                format!("пробовал: {}", tried.join("; "))
            }
        )
    }

    /// Диагностический дамп: какие приложения отдают AT-SPI, что у них в фокусе,
    /// и сколько времени занял каждый этап.
    pub async fn dump(app_hint: Option<String>, max_depth: usize, verbose: bool) -> Result<String> {
        let mut timer = Timer::new();
        let bus = Bus::connect(&mut timer).await?;
        let apps = bus.apps().await?;
        timer.mark("приложения");

        let hint = app_hint.map(|h| h.to_lowercase());
        let mut out = String::new();
        let mut shown = 0usize;
        let mut visited = 0usize;

        for (dest, app_name) in apps {
            if let Some(h) = &hint {
                if !app_name.to_lowercase().contains(h.as_str()) {
                    continue;
                }
            }
            shown += 1;
            out.push_str(&format!("приложение {dest} ({app_name:?})\n"));
            if verbose {
                println!("приложение {dest} ({app_name:?}):");
            }
            let (focused, walked) = find_focused(&bus, &dest, max_depth, verbose).await?;
            visited += walked;
            timer.mark(format!("обход {app_name}"));
            out.push_str(&format!("  объектов пройдено: {walked}\n"));
            out.push_str(&format!("  focused: {}\n", focused.len()));
            for f in focused {
                out.push_str(&format!(
                    "    {} {} роль={} name={:?} EditableText: {}\n",
                    f.dest,
                    f.path,
                    f.role,
                    f.name,
                    if f.editable { "да" } else { "нет" }
                ));
            }
        }
        if shown == 0 {
            out.push_str(
                "приложений доступности не найдено: ни одно приложение не отдало \
                 свой корневой объект реестру (проверь QT_ACCESSIBILITY=1 / \
                 --force-renderer-accessibility)\n",
            );
        }
        out.push_str(&format!(
            "этапы: {} | объектов пройдено: {visited} | D-Bus-вызовов: {}\n",
            timer.line(),
            bus.calls.load(Ordering::Relaxed)
        ));
        Ok(out)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Фильтр по роли — то, что не даёт писать в фокусный документ вместо поля ввода.
        #[test]
        fn input_roles_allowed() {
            for role in [Role::Entry, Role::PasswordText, Role::ComboBox, Role::Text] {
                assert!(looks_like_input_field(role), "{role} должно допускаться");
            }
            for role in [
                Role::DocumentWeb,
                Role::DocumentText,
                Role::Paragraph,
                Role::Frame,
                Role::Unknown,
            ] {
                assert!(!looks_like_input_field(role), "{role} не поле ввода");
            }
        }
    }
}

/// Вставка через AT-SPI2. Возвращает адрес объекта, куда попало слово.
///
/// Обход дерева — это сотни D-Bus-_round trip'ов, поэтому всё выполняется в
/// отдельном потоке с жёстким дедлайном: зависшее приложение не должно
/// блокировать демон.
/// Вставка с дефолтной защитой по роли (только поля ввода).
#[cfg(feature = "atspi")]
pub fn insert_text(text: &str, app_hint: Option<&str>) -> Result<String> {
    insert_text_with(text, app_hint, false)
}

/// Вставка с `allow_any_role = true` пишет и в нестандартные редактируемые узлы
/// (диагностика: некоторые приложения помечают `focused` не поле, а документ).
#[cfg(feature = "atspi")]
pub fn insert_text_with(
    text: &str,
    app_hint: Option<&str>,
    allow_any_role: bool,
) -> Result<String> {
    let text = text.to_string();
    let hint = app_hint.map(|h| h.to_string());
    with_deadline(Duration::from_secs(8), move || {
        Box::pin(imp::insert(text, hint, allow_any_role))
    })
}

/// Дерево доступности для диагностики (`dictactl atspi-dump`).
/// `tree = true` печатает само дерево (шумно, но видно, где обход упирается в лимит).
#[cfg(feature = "atspi")]
pub fn dump(app_hint: Option<&str>, max_depth: usize, tree: bool) -> Result<String> {
    let hint = app_hint.map(|h| h.to_string());
    with_deadline(Duration::from_secs(20), move || {
        Box::pin(imp::dump(hint, max_depth, tree))
    })
}

#[cfg(feature = "atspi")]
fn with_deadline(
    limit: std::time::Duration,
    task: impl FnOnce() -> futures::future::BoxFuture<'static, Result<String>> + Send + 'static,
) -> Result<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let res = futures::executor::block_on(task());
        let _ = tx.send(res);
    });
    rx.recv_timeout(limit)
        .map_err(|_| anyhow::anyhow!("AT-SPI: нет ответа за {} с", limit.as_secs()))?
}

#[cfg(not(feature = "atspi"))]
pub fn insert_text(_text: &str, _app_hint: Option<&str>) -> Result<String> {
    anyhow::bail!(
        "AT-SPI недоступен: демон собран без фичи atspi \
         (cargo build --features dictad/atspi,dictactl/atspi)"
    )
}

#[cfg(not(feature = "atspi"))]
pub fn insert_text_with(
    _text: &str,
    _app_hint: Option<&str>,
    _allow_any_role: bool,
) -> Result<String> {
    anyhow::bail!(
        "AT-SPI недоступен: демон собран без фичи atspi \
         (cargo build --features dictad/atspi,dictactl/atspi)"
    )
}

#[cfg(not(feature = "atspi"))]
pub fn dump(_app_hint: Option<&str>, _max_depth: usize, _tree: bool) -> Result<String> {
    anyhow::bail!("AT-SPI недоступно: собрано без фичи atspi");
}
