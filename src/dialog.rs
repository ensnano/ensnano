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

fn filter_has_extension(dialog_filters: DialogFilters, extension: &str) -> bool {
    dialog_filters
        .iter()
        .any(|df| df.extensions.contains(&extension))
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
    let mut dialog = rfd::AsyncFileDialog::new();

    let default_extension = dialog_filters
        .first()
        .and_then(|f| f.extensions.first().copied());

    let starting_name = starting_name.and_then(|p| {
        let mut path_buf = PathBuf::from(p.as_ref());
        let extension = path_buf.extension();
        if extension.is_none() && default_extension.is_some() {
            path_buf.set_extension(default_extension.unwrap());
        } else if let Some(current_extension) = extension
            && !filter_has_extension(
                dialog_filters,
                current_extension.to_str().unwrap_or_default(),
            )
        {
            let new_extension = default_extension.unwrap_or_default();
            path_buf.set_extension(new_extension);
        }
        path_buf.file_name().map(OsStr::to_os_string)
    });

    log::info!("starting name filtered {starting_name:?}");
    for filter in dialog_filters {
        dialog = dialog.add_filter(filter.name, filter.extensions);
    }
    log::info!(
        "starting path filtered {:?}",
        starting_path.as_ref().map(AsRef::as_ref)
    );
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
            let result = file.map(|handle| {
                let mut path_buf: PathBuf = handle.path().into();
                let extension = path_buf.extension();
                if extension.is_none() && default_extension.is_some() {
                    path_buf.set_extension(default_extension.unwrap());
                } else if let Some(current_extension) = extension
                    && !filter_has_extension(
                        dialog_filters,
                        current_extension.to_str().unwrap_or_default(),
                    )
                {
                    let new_extension = format!(
                        "{}.{}",
                        current_extension.to_str().unwrap(),
                        default_extension.unwrap_or_default()
                    );
                    path_buf.set_extension(new_extension);
                }
                path_buf
            });
            log_err![snd.send(result)];
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
