use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

pub struct Frame {
    pub data: Vec<u8>,
    pub size: [u32; 2],
}

pub struct FrameRecorder {
    sender: Sender<Frame>,
    receiver: Arc<Mutex<Receiver<Frame>>>,
    encoding_thread: Option<thread::JoinHandle<()>>,
}

impl FrameRecorder {
    pub fn new() -> Self {
        let (sender, reciever) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(reciever));

        Self {
            sender,
            receiver,
            encoding_thread: None,
        }
    }

    pub fn start_encoding_thread(&mut self) {
        let receiver = Arc::clone(&self.receiver);
        self.encoding_thread = Some(thread::spawn(move || {
            let receiver = receiver.lock().unwrap();
            while let Ok(frame) = receiver.recv() {
                println!("Encoding buffer of size {:?}", frame.size);
            }
        }))
    }

    pub fn send_frame(&self, frame: Frame) {
        self.sender.send(frame).expect("Failed to send frame")
    }
}
