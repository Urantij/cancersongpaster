use rdev::{listen, Event, Key};
use std::cmp::PartialEq;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

#[derive(PartialEq)]
pub enum KeyActionType {
    Release,
    Press,
}

pub struct KeyEvent {
    pub key: Key,
    pub is_ctrl: bool,
    pub action_type: KeyActionType,
}

struct ControlContainer {
    is_ctrl: bool,
    is_pressed: bool,
}

pub struct ListenControl {
    hidden_control: Arc<Mutex<HiddenControl>>,
    pub receiver: mpsc::Receiver<KeyEvent>,
}

struct HiddenControl {
    listen_handle: Option<JoinHandle<()>>,
}

impl HiddenControl {
    pub fn stop(&mut self) {
        if self.listen_handle.is_none() {
            return;
        }

        self.listen_handle.take().unwrap();
    }
}

impl ListenControl {
    pub fn create(key: Key) -> ListenControl {
        let hidden = Arc::new(Mutex::new(HiddenControl {
            listen_handle: None,
        }));

        let (sender, receiver) = mpsc::channel();

        let hidden_listen = hidden.clone();

        let handle = thread::spawn(move || {
            let hidden_callback = hidden_listen.clone();

            let mut container = ControlContainer {
                is_ctrl: false,
                is_pressed: false,
            };

            let _ = listen(move |event| {
                ListenControl::process_event(
                    key,
                    &event,
                    &sender,
                    &mut container,
                    &hidden_callback,
                );
            });

            let mut locked = hidden_listen.lock().unwrap();
            locked.stop();
        });

        hidden.lock().unwrap().listen_handle = Some(handle);

        ListenControl {
            hidden_control: hidden,
            receiver,
        }
    }

    pub fn stop(self) {
        self.hidden_control.lock().unwrap().stop();
    }

    fn process_event(
        key: Key,
        event: &Event,
        sender: &mpsc::Sender<KeyEvent>,
        mut container: &mut ControlContainer,
        hidden_callback: &Arc<Mutex<HiddenControl>>,
    ) {
        if event.event_type == rdev::EventType::KeyPress(Key::ControlLeft) {
            container.is_ctrl = true;
        } else if event.event_type == rdev::EventType::KeyRelease(Key::ControlLeft) {
            container.is_ctrl = false;
        }

        if event.event_type == rdev::EventType::KeyPress(key) {
            if container.is_pressed {
                return;
            }
            container.is_pressed = true;
            return;
        } else if event.event_type == rdev::EventType::KeyRelease(key) {
            if !container.is_pressed {
                return;
            }
            container.is_pressed = false;
        } else {
            return;
        }

        let result = KeyEvent {
            key,
            is_ctrl: container.is_ctrl,
            action_type: match container.is_pressed {
                true => KeyActionType::Press,
                false => KeyActionType::Release,
            },
        };

        if sender.send(result).is_err() {
            let mut locked = hidden_callback.lock().unwrap();
            locked.stop();
        }
    }
}
