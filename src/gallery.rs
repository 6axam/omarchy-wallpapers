use crate::shared;
use gdk::DragAction;
use gdk_pixbuf::{InterpType, Pixbuf};
use glib::ControlFlow;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, DestDefaults,
    FileChooserAction, FileChooserNative, FileFilter, FlowBox, Image, Label, Orientation,
    ResponseType, ScrolledWindow, SelectionMode, StyleContext, TargetEntry, TargetFlags,
};
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc, time::Duration};

struct View {
    flow: FlowBox,
    timer: Label,
    status: Label,
    files: Vec<PathBuf>,
    current: Option<PathBuf>,
    window: ApplicationWindow,
}

pub fn run() -> Result<(), String> {
    let app = Application::new(Some("io.github.wallpapers.gallery"), Default::default());
    app.connect_activate(build);
    app.run_with_args::<&str>(&[]);
    Ok(())
}
fn build(app: &Application) {
    let css = CssProvider::new();
    let _=css.load_from_data(b"window { background:#11131a; color:#eef0f6; } .header { font-size:22px; font-weight:bold; } .muted { color:#a7acbb; } button.tile { padding:0; border:3px solid transparent; border-radius:3px; background:#1e222d; } button.tile.current, button.add:hover { border-color:#a0afff; } button.add { color:#a7acbb; background:#1e222d; } .plus { font-size:44px; } flowboxchild { padding:4px; } button.action { padding:7px 14px; }");
    if let Some(screen) = gdk::Screen::default() {
        StyleContext::add_provider_for_screen(
            &screen,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Мои обои")
        .default_width(960)
        .default_height(660)
        .build();
    let outer = GtkBox::new(Orientation::Vertical, 8);
    outer.set_border_width(20);
    let top = GtkBox::new(Orientation::Horizontal, 10);
    let title = Label::new(Some("Мои обои"));
    title.style_context().add_class("header");
    title.set_halign(gtk::Align::Start);
    title.set_hexpand(true);
    let shuffle = Button::with_label("⤨  Перемешать");
    shuffle.style_context().add_class("action");
    top.pack_start(&title, true, true, 0);
    top.pack_end(&shuffle, false, false, 0);
    outer.pack_start(&top, false, false, 0);
    let timer = Label::new(Some("Подключение к таймеру…"));
    timer.set_halign(gtk::Align::Start);
    outer.pack_start(&timer, false, false, 0);
    let status = Label::new(Some("Нажми на обои, чтобы установить"));
    status.set_halign(gtk::Align::Start);
    status.style_context().add_class("muted");
    outer.pack_start(&status, false, false, 0);
    let flow = FlowBox::new();
    flow.set_selection_mode(SelectionMode::None);
    flow.set_max_children_per_line(20);
    flow.set_min_children_per_line(1);
    flow.set_homogeneous(false);
    flow.set_halign(gtk::Align::Start);
    flow.set_valign(gtk::Align::Start);
    let scroll = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.add(&flow);
    outer.pack_start(&scroll, true, true, 0);
    window.add(&outer);
    let view = Rc::new(RefCell::new(View {
        flow,
        timer,
        status,
        files: vec![],
        current: None,
        window: window.clone(),
    }));
    {
        let v = Rc::clone(&view);
        shuffle.connect_clicked(move |_| match shared::shuffle() {
            Ok(()) => v
                .borrow()
                .status
                .set_text("Очередь перемешана · время следующей смены обновлено"),
            Err(e) => show_error(&v.borrow().window, &e),
        });
    }
    refresh(&view, true);
    {
        let v = Rc::clone(&view);
        glib::timeout_add_local(Duration::from_secs(1), move || {
            refresh(&v, false);
            ControlFlow::Continue
        });
    }
    window.show_all();
}
fn refresh(view: &Rc<RefCell<View>>, force: bool) {
    let current = shared::current();
    let files = shared::config()
        .map(|c| shared::images(&shared::folder(&c)))
        .unwrap_or_default();
    let rebuild = {
        let v = view.borrow();
        force || v.files != files || v.current != current
    };
    if rebuild {
        let mut v = view.borrow_mut();
        v.files = files;
        v.current = current;
        drop(v);
        draw(view);
    }
    let text = fs::read(shared::state_path())
        .ok()
        .and_then(|b| serde_json::from_slice::<shared::State>(&b).ok())
        .and_then(|s| {
            let process = fs::read(format!("/proc/{}/cmdline", s.pid)).ok()?;
            if !process
                .windows(b"omarchy-wallpapers".len())
                .any(|w| w == b"omarchy-wallpapers")
            {
                return None;
            }
            Some(match s.status.as_str() {
                "scheduled" => {
                    let seconds =
                        (s.deadline.unwrap_or(0.0) - shared::now()).ceil().max(0.0) as u64;
                    if seconds == 0 {
                        "Смена обоев…".to_string()
                    } else if seconds >= 3600 {
                        format!(
                            "Следующая смена через {}:{:02}:{:02}",
                            seconds / 3600,
                            (seconds / 60) % 60,
                            seconds % 60
                        )
                    } else {
                        format!(
                            "Следующая смена через {:02}:{:02}",
                            seconds / 60,
                            seconds % 60
                        )
                    }
                }
                "empty" => "Ожидание картинок в очереди".into(),
                _ => "Таймер недоступен: ошибка службы".into(),
            })
        })
        .unwrap_or_else(|| "Таймер недоступен: служба остановлена".into());
    view.borrow().timer.set_text(&text);
}
fn draw(view: &Rc<RefCell<View>>) {
    let v = view.borrow();
    for child in v.flow.children() {
        v.flow.remove(&child);
    }
    for path in &v.files {
        let pix = match Pixbuf::from_file_at_scale(path, 260, 146, true) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let width = pix.width();
        let height = pix.height();
        let scale = (260.0 / width as f64).max(146.0 / height as f64);
        let scaled = pix
            .scale_simple(
                (width as f64 * scale).ceil() as i32,
                (height as f64 * scale).ceil() as i32,
                InterpType::Bilinear,
            )
            .unwrap_or(pix);
        let cropped = scaled.new_subpixbuf(
            (scaled.width() - 260) / 2,
            (scaled.height() - 146) / 2,
            260,
            146,
        );
        let picture = Image::from_pixbuf(Some(&cropped));
        let button = Button::new();
        button.style_context().add_class("tile");
        if fs::canonicalize(path).ok() == v.current {
            button.style_context().add_class("current");
        }
        button.add(&picture);
        let p = path.clone();
        let weak = Rc::downgrade(view);
        button.connect_clicked(move |_| {
            if let Some(v) = weak.upgrade() {
                match shared::apply(&p) {
                    Ok(()) => {
                        v.borrow().status.set_text(
                            "Обои установлены · следующая автоматическая смена по расписанию",
                        );
                        refresh(&v, true);
                    }
                    Err(e) => show_error(&v.borrow().window, &e),
                }
            }
        });
        v.flow.add(&button);
    }
    let add = Button::new();
    add.style_context().add_class("tile");
    add.style_context().add_class("add");
    add.set_size_request(266, 152);
    let content = GtkBox::new(Orientation::Vertical, 0);
    content.set_valign(gtk::Align::Center);
    let plus = Label::new(Some("+"));
    plus.style_context().add_class("plus");
    content.pack_start(&plus, false, false, 0);
    content.pack_start(&Label::new(Some("Добавить обои")), false, false, 0);
    add.add(&content);
    let weak = Rc::downgrade(view);
    add.connect_clicked(move |_| {
        if let Some(v) = weak.upgrade() {
            choose(&v);
        }
    });
    add.drag_dest_set(
        DestDefaults::ALL,
        &[TargetEntry::new("text/uri-list", TargetFlags::OTHER_APP, 0)],
        DragAction::COPY,
    );
    let weak = Rc::downgrade(view);
    add.connect_drag_data_received(move |_, ctx, _, _, data, _, time| {
        let paths: Vec<PathBuf> = data
            .uris()
            .iter()
            .filter_map(|uri| glib::filename_from_uri(uri).ok().map(|x| x.0))
            .collect();
        let ok = !paths.is_empty();
        if let Some(v) = weak.upgrade() {
            import(&v, &paths);
        }
        ctx.drag_finish(ok, false, time);
    });
    v.flow.add(&add);
    v.flow.show_all();
}
fn choose(view: &Rc<RefCell<View>>) {
    let dialog = FileChooserNative::new(
        Some("Добавить обои"),
        Some(&view.borrow().window),
        FileChooserAction::Open,
        Some("Открыть"),
        Some("Отмена"),
    );
    dialog.set_select_multiple(true);
    let filter = FileFilter::new();
    filter.set_name(Some("Изображения"));
    filter.add_pixbuf_formats();
    dialog.add_filter(filter);
    let weak = Rc::downgrade(view);
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(v) = weak.upgrade() {
                import(&v, &dialog.filenames());
            }
        }
        dialog.destroy();
    });
    dialog.show();
}
fn import(view: &Rc<RefCell<View>>, paths: &[PathBuf]) {
    let folder = match shared::config() {
        Ok(c) => shared::folder(&c),
        Err(e) => {
            show_error(&view.borrow().window, &e);
            return;
        }
    };
    if let Err(e) = fs::create_dir_all(&folder) {
        show_error(&view.borrow().window, &e.to_string());
        return;
    }
    let mut added = 0;
    let mut errors = vec![];
    for path in paths {
        if !path.is_file()
            || !matches!(
                path.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .as_str(),
                "jpg" | "jpeg" | "png" | "webp" | "bmp"
            )
            || Pixbuf::file_info(path).is_none()
        {
            errors.push(format!("{}: неподдерживаемое изображение", path.display()));
            continue;
        }
        let Some(name) = path.file_name() else {
            continue;
        };
        let mut dest = folder.join(name);
        if fs::canonicalize(path).ok() == fs::canonicalize(&dest).ok() && dest.exists() {
            continue;
        }
        if dest.exists() {
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = path.extension().unwrap_or_default().to_string_lossy();
            let mut n = 1;
            while dest.exists() {
                dest = folder.join(format!("{stem}-{n}.{ext}"));
                n += 1;
            }
        }
        let temp = folder.join(format!(
            ".wallpaper-import-{}-{added}.tmp",
            std::process::id()
        ));
        match fs::copy(path, &temp).and_then(|_| fs::rename(&temp, &dest)) {
            Ok(()) => added += 1,
            Err(e) => {
                let _ = fs::remove_file(&temp);
                errors.push(format!("{}: {e}", path.display()));
            }
        }
    }
    view.borrow()
        .status
        .set_text(&format!("Добавлено обоев: {added}"));
    refresh(view, true);
    if !errors.is_empty() {
        show_error(&view.borrow().window, &errors.join("\n"));
    }
}
fn show_error(window: &ApplicationWindow, message: &str) {
    let d = gtk::MessageDialog::new(
        Some(window),
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Close,
        message,
    );
    d.connect_response(|d, _| d.close());
    d.show();
}
