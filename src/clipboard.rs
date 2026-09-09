//! Event-driven Wayland data-control capture. No polling, shell or simulated keys.
use crate::model::{Clip, MAX_CLIP_BYTES, now};
use std::{
    collections::HashMap,
    io::Read,
    os::{fd::AsFd, unix::net::UnixStream},
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    time::{Duration, Instant},
};
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, delegate_noop,
    globals::{GlobalListContents, registry_queue_init},
    protocol::{wl_registry::WlRegistry, wl_seat::WlSeat},
};
use wayland_protocols::ext::data_control::v1::client::{
    ext_data_control_device_v1 as ed, ext_data_control_manager_v1 as em,
    ext_data_control_offer_v1 as eo,
};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1 as wd, zwlr_data_control_manager_v1 as wm,
    zwlr_data_control_offer_v1 as wo,
};

#[derive(Debug)]
pub enum Event {
    Clip(Clip, u64),
    Status(Result<(), String>),
    Rejected(String),
}

pub struct Monitor {
    pub events: Receiver<Event>,
    /// Odd = paused. Incrementing invalidates in-flight transfers as well.
    pub epoch: Arc<AtomicU64>,
}
impl Monitor {
    pub fn idle() -> Self {
        let (_, events) = sync_channel(1);
        Self {
            events,
            epoch: Arc::new(AtomicU64::new(1)),
        }
    }
    pub fn start(paused: bool) -> Self {
        let (sender, events) = sync_channel(32);
        let epoch = Arc::new(AtomicU64::new(u64::from(paused)));
        let worker_epoch = epoch.clone();
        std::thread::spawn(move || {
            loop {
                if let Err(error) = watch(sender.clone(), worker_epoch.clone())
                    && sender.send(Event::Status(Err(error))).is_err()
                {
                    break;
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        });
        Self { events, epoch }
    }
    pub fn paused(&self) -> bool {
        self.epoch.load(Ordering::SeqCst) % 2 == 1
    }
    pub fn toggle_pause(&self) {
        self.epoch.fetch_add(1, Ordering::SeqCst);
    }
    pub fn invalidate(&self) {
        self.epoch.fetch_add(2, Ordering::SeqCst);
    }
}

struct State {
    offers: HashMap<wayland_client::backend::ObjectId, Vec<String>>,
    sender: SyncSender<Event>,
    epoch: Arc<AtomicU64>,
    transfers: Arc<AtomicUsize>,
    finished: bool,
}

fn watch(sender: SyncSender<Event>, epoch: Arc<AtomicU64>) -> Result<(), String> {
    let connection = Connection::connect_to_env().map_err(|e| e.to_string())?;
    let (globals, mut queue) =
        registry_queue_init::<State>(&connection).map_err(|e| e.to_string())?;
    let qh = queue.handle();
    let seat: WlSeat = globals.bind(&qh, 1..=7, ()).map_err(|e| e.to_string())?;
    // Prefer the standardized protocol; older COSMIC versions expose wlr-data-control.
    let _device = if let Ok(manager) =
        globals.bind::<em::ExtDataControlManagerV1, _, _>(&qh, 1..=1, ())
    {
        let device = manager.get_data_device(&seat, &qh, ());
        manager.destroy();
        Device::Ext(device)
    } else {
        let manager: wm::ZwlrDataControlManagerV1 = globals.bind(&qh, 1..=2, ()).map_err(|_| "Ce compositeur n’expose pas data-control. Lance l’applet dans COSMIC, hors Flatpak.".to_string())?;
        let device = manager.get_data_device(&seat, &qh, ());
        manager.destroy();
        Device::Wlr(device)
    };
    let mut state = State {
        offers: HashMap::new(),
        sender,
        epoch,
        transfers: Arc::new(AtomicUsize::new(0)),
        finished: false,
    };
    queue.roundtrip(&mut state).map_err(|e| e.to_string())?;
    let _ = state.sender.send(Event::Status(Ok(())));
    while !state.finished {
        queue
            .blocking_dispatch(&mut state)
            .map_err(|e| e.to_string())?;
    }
    Err("Connexion au presse-papiers interrompue ; reconnexion…".into())
}

#[allow(dead_code)]
enum Device {
    Ext(ed::ExtDataControlDeviceV1),
    Wlr(wd::ZwlrDataControlDeviceV1),
}

pub fn select_mime(types: &[String]) -> Option<String> {
    if types.iter().any(|m| {
        [
            "x-kde-passwordManagerHint",
            "application/x-keepassxc-secret",
            "application/x-nspasteboard-concealed-type",
        ]
        .contains(&m.as_str())
    }) {
        return None;
    }
    [
        "text/uri-list",
        "image/png",
        "image/jpeg",
        "text/plain;charset=utf-8",
        "text/plain;charset=UTF-8",
        "text/plain",
        "UTF8_STRING",
    ]
    .into_iter()
    .find(|m| types.iter().any(|t| t == m))
    .map(str::to_string)
}

impl State {
    fn receive(
        &mut self,
        id: wayland_client::backend::ObjectId,
        request: impl FnOnce(String, &UnixStream),
        connection: &Connection,
    ) {
        let types = self.offers.remove(&id).unwrap_or_default();
        let epoch = self.epoch.load(Ordering::SeqCst);
        if epoch % 2 == 1 {
            return;
        }
        let Some(mime) = select_mime(&types) else {
            return;
        };
        // Bound memory and worker count even when a source produces many offers.
        if self
            .transfers
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                (n < 4).then_some(n + 1)
            })
            .is_err()
        {
            return;
        }
        let Ok((mut read, write)) = UnixStream::pair() else {
            self.transfers.fetch_sub(1, Ordering::SeqCst);
            return;
        };
        request(mime.clone(), &write);
        drop(write);
        if connection.flush().is_err() {
            self.transfers.fetch_sub(1, Ordering::SeqCst);
            return;
        }
        let timestamp = now();
        let sender = self.sender.clone();
        let control = self.epoch.clone();
        let active = self.transfers.clone();
        std::thread::spawn(move || {
            let result = read_clip(&mut read).and_then(|bytes| {
                let mime = if mime.starts_with("text/plain") || mime == "UTF8_STRING" {
                    "text/plain;charset=utf-8".into()
                } else {
                    mime
                };
                Clip::new(mime, bytes, timestamp)
            });
            if control.load(Ordering::SeqCst) == epoch {
                let event = match result {
                    Ok(clip) => Event::Clip(clip, epoch),
                    Err(e) => Event::Rejected(e),
                };
                let _ = sender.send(event);
            }
            active.fetch_sub(1, Ordering::SeqCst);
        });
    }
}

fn read_clip(read: &mut UnixStream) -> Result<Vec<u8>, String> {
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut bytes = Vec::new();
    let mut buffer = [0; 65536];
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or("Lecture du presse-papiers trop lente")?;
        read.set_read_timeout(Some(remaining))
            .map_err(|e| e.to_string())?;
        let count = read
            .read(&mut buffer)
            .map_err(|e| format!("Lecture interrompue : {e}"))?;
        if count == 0 {
            return Ok(bytes);
        }
        if bytes.len() + count > MAX_CLIP_BYTES {
            return Err("Copie ignorée : taille supérieure à 16 Mio".into());
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}

macro_rules! protocol {
    ($device:ty, $de:ident, $offer:ty, $oe:ident) => {
        impl Dispatch<$device, ()> for State {
            fn event(state: &mut Self, _: &$device, event: <$device as Proxy>::Event, _: &(), connection: &Connection, _: &QueueHandle<Self>) {
                match event {
                    $de::Event::DataOffer { id } => { state.offers.insert(id.id(), Vec::new()); }
                    $de::Event::Selection { id: Some(offer) } => {
                        state.receive(offer.id(), |mime, fd| offer.receive(mime, fd.as_fd()), connection);
                        offer.destroy();
                    }
                    $de::Event::PrimarySelection { id: Some(offer) } => { state.offers.remove(&offer.id()); offer.destroy(); }
                    $de::Event::Finished => { state.finished = true; }
                    _ => {}
                }
            }
            wayland_client::event_created_child!(State, $device, [0 => ($offer, ())]);
        }
        impl Dispatch<$offer, ()> for State {
            fn event(state: &mut Self, offer: &$offer, event: <$offer as Proxy>::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
                if let $oe::Event::Offer { mime_type } = event { state.offers.entry(offer.id()).or_default().push(mime_type); }
            }
        }
    }
}
protocol!(
    ed::ExtDataControlDeviceV1,
    ed,
    eo::ExtDataControlOfferV1,
    eo
);
protocol!(
    wd::ZwlrDataControlDeviceV1,
    wd,
    wo::ZwlrDataControlOfferV1,
    wo
);
delegate_noop!(State: ignore WlSeat);
delegate_noop!(State: ignore em::ExtDataControlManagerV1);
delegate_noop!(State: ignore wm::ZwlrDataControlManagerV1);
impl Dispatch<WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: <WlRegistry as Proxy>::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

pub fn copy(clip: Clip) -> Result<(), String> {
    use wl_clipboard_rs::copy::{MimeType, Options, Source};
    Options::new()
        .copy(
            Source::Bytes(clip.bytes.into_boxed_slice()),
            MimeType::Specific(clip.mime),
        )
        .map_err(|e| e.to_string())
}
