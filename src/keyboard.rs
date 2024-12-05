use rdev::{listen, Key};
use std::cmp::PartialEq;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

#[derive(PartialEq)]
pub enum KeyActionType {
    Release,
    Press,
}

pub struct ListenControl {
    hidden_control: Arc<Mutex<HiddenControl>>,
}

struct HiddenControl {
    listen_handle: Option<JoinHandle<()>>,
}

struct ControlContainer {
    is_ctrl: bool,
    is_pressed: bool,
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
    pub fn create(
        sender: mpsc::Sender<()>,
        key: Key,
        with_ctrl: bool,
        key_action_type: KeyActionType,
    ) -> ListenControl {
        let hidden = Arc::new(Mutex::new(HiddenControl {
            listen_handle: None,
        }));

        let hidden_listen = hidden.clone();

        let handle = thread::spawn(move || {
            let hidden_callback = hidden_listen.clone();

            let mut container = ControlContainer {
                is_ctrl: false,
                is_pressed: false,
            };

            let _ = listen(move |event| {
                let mut send = false;

                if event.event_type == rdev::EventType::KeyPress(Key::ControlLeft) {
                    container.is_ctrl = true;
                } else if event.event_type == rdev::EventType::KeyRelease(Key::ControlLeft) {
                    container.is_ctrl = false;
                } else if event.event_type == rdev::EventType::KeyPress(key) {
                    if container.is_pressed {
                        return;
                    }
                    container.is_pressed = true;

                    send = key_action_type == KeyActionType::Press;
                } else if event.event_type == rdev::EventType::KeyRelease(key) {
                    if !container.is_pressed {
                        return;
                    }
                    container.is_pressed = false;

                    send = key_action_type == KeyActionType::Release;
                }

                if !send {
                    return;
                }

                if with_ctrl && !container.is_ctrl {
                    return;
                }

                if sender.send(()).is_err() {
                    let mut locked = hidden_callback.lock().unwrap();
                    locked.stop();
                    return;
                }
            });

            let mut locked = hidden_listen.lock().unwrap();
            locked.stop();
        });

        hidden.lock().unwrap().listen_handle = Some(handle);

        ListenControl {
            hidden_control: hidden,
        }
    }

    pub fn stop(self) {
        self.hidden_control.lock().unwrap().stop();
    }
}
