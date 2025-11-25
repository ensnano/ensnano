//! Functions that create [states](`crate::controller::State`) in which the user is interacting
//! with a dialog box (alert, file picker, ...).

use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};

macro_rules! log_err {
    ($x:expr) => {
        if $x.is_err() {
            log::error!("Unexpected error")
        }
    };
}

pub struct DialogFilter {
    name: &'static str,
    extensions: &'static [&'static str],
}
impl DialogFilter {
    pub const fn new(name: &'static str, extensions: &'static [&'static str]) -> Self {
        Self { name, extensions }
    }
}

pub type DialogFilters = &'static [DialogFilter];

/// A question to which the user must answer yes or no
pub struct YesNoQuestion(mpsc::Receiver<bool>);
impl YesNoQuestion {
    pub fn answer(&self) -> Option<bool> {
        self.0.try_recv().ok()
    }
}

pub fn yes_no_dialog(message: Cow<'static, str>) -> YesNoQuestion {
    let msg = rfd::AsyncMessageDialog::new()
        .set_description(message.as_ref())
        .set_buttons(rfd::MessageButtons::YesNo)
        .show();
    let (snd, rcv) = mpsc::channel();
    thread::spawn(move || {
        let choice = async move {
            log::debug!("thread spawned");
            let ret = msg.await == rfd::MessageDialogResult::Yes;
            log::debug!("about to send");
            log_err![snd.send(ret)];
        };
        futures::executor::block_on(choice);
    });
    YesNoQuestion(rcv)
}

/// A message that the user must acknowledge
pub struct MustAckMessage(mpsc::Receiver<()>);
impl MustAckMessage {
    pub fn was_ack(&self) -> bool {
        self.0.try_recv().is_ok()
    }
}

pub fn blocking_message(message: Cow<'static, str>, level: rfd::MessageLevel) -> MustAckMessage {
    let msg = rfd::AsyncMessageDialog::new()
        .set_level(level)
        .set_description(message.as_ref())
        .show();
    let (snd, rcv) = mpsc::channel();
    thread::spawn(move || {
        futures::executor::block_on(msg);
        log_err!(snd.send(()));
    });
    MustAckMessage(rcv)
}

pub struct PathInput(mpsc::Receiver<Option<PathBuf>>);
impl PathInput {
    pub fn get(&self) -> Option<Option<PathBuf>> {
        self.0.try_recv().ok()
    }
}

/// Normalize a path's extension according to the dialog filters and default extension.
fn normalize_extension(
    path: &mut PathBuf,
    dialog_filters: DialogFilters,
    default_extension: Option<&str>,
    append_on_mismatch: bool,
) {
    // If we don't have a default extension, there's nothing to normalize to.
    let Some(default) = default_extension else {
        return;
    };

    let current_extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .filter(|ext| !ext.is_empty());

    match current_extension {
        None => {
            path.set_extension(default);
        }
        Some(current) => {
            let is_known_extension = dialog_filters
                .iter()
                .any(|df| df.extensions.contains(&current));
            if !is_known_extension {
                if append_on_mismatch {
                    path.set_extension(format!("{current}.{default}"));
                } else {
                    path.set_extension(default);
                }
            }
        }
    }
}

pub fn get_file_to_write(
    dialog_filters: DialogFilters,
    starting_path: Option<impl AsRef<Path>>,
    starting_name: Option<impl AsRef<Path>>,
) -> PathInput {
    log::info!(
        "starting path {:?}",
        starting_path.as_ref().and_then(|p| p.as_ref().to_str())
    );
    log::info!(
        "starting name {:?}",
        starting_name.as_ref().and_then(|p| p.as_ref().to_str())
    );

    let default_extension = dialog_filters
        .first()
        .and_then(|f| f.extensions.first().copied());

    let starting_name = starting_name.and_then(|name| {
        let mut path_buf = PathBuf::from(name.as_ref());
        normalize_extension(&mut path_buf, dialog_filters, default_extension, false);
        path_buf.file_name().map(OsStr::to_os_string)
    });

    log::info!(
        "starting path filtered {:?}",
        starting_path.as_ref().map(AsRef::as_ref)
    );
    log::info!("starting name filtered {starting_name:?}");

    let mut dialog = rfd::AsyncFileDialog::new();
    for filter in dialog_filters {
        dialog = dialog.add_filter(filter.name, filter.extensions);
    }
    if let Some(path) = starting_path {
        dialog = dialog.set_directory(path);
    }
    if let Some(name) = starting_name {
        dialog = dialog.set_file_name(&*name.to_string_lossy());
    }

    let future_file = dialog.save_file();
    let (snd, rcv) = mpsc::channel();

    thread::spawn(move || {
        let save_op = async move {
            let file = future_file.await;

            let path_buf = file.map(|handle| {
                let mut path_buf: PathBuf = handle.path().into();
                normalize_extension(&mut path_buf, dialog_filters, default_extension, true);
                path_buf
            });

            log_err![snd.send(path_buf)];
        };

        futures::executor::block_on(save_op);
    });

    PathInput(rcv)
}

pub fn load<P: AsRef<Path>>(starting_path: Option<P>, dialog_filters: DialogFilters) -> PathInput {
    let mut dialog = rfd::AsyncFileDialog::new();
    for dialog_filter in dialog_filters {
        dialog = dialog.add_filter(dialog_filter.name, dialog_filter.extensions);
    }
    log::info!(
        "starting path {:?}",
        starting_path.as_ref().map(AsRef::as_ref)
    );
    if let Some(path) = starting_path {
        dialog = dialog.set_directory(path);
    }
    let future_file = dialog.pick_file();
    let (snd, rcv) = mpsc::channel();
    thread::spawn(move || {
        let load_op = async move {
            let file = future_file.await;
            if let Some(handle) = file {
                let path_buf: PathBuf = handle.path().into();
                log_err![snd.send(Some(path_buf))];
            } else {
                log_err![snd.send(None)];
            }
        };
        futures::executor::block_on(load_op);
    });
    PathInput(rcv)
}
