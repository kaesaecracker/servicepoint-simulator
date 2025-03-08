use crate::command_executor::CommandExecute;
use crate::{
    command_executor::{CommandExecutionContext, ExecutionResult},
    gui::AppEvents,
};
use log::{debug, error, warn};
use servicepoint::TypedCommand;
use std::{
    io::ErrorKind, net::UdpSocket, sync::mpsc::Receiver, time::Duration,
};
use winit::event_loop::EventLoopProxy;

const BUF_SIZE: usize = 8985;

#[derive(Debug)]
pub struct UdpServer<'t> {
    socket: UdpSocket,
    stop_rx: Receiver<()>,
    command_executor: CommandExecutionContext<'t>,
    app_events: EventLoopProxy<AppEvents>,
    buf: [u8; BUF_SIZE],
}

impl<'t> UdpServer<'t> {
    pub fn new(
        bind: String,
        stop_rx: Receiver<()>,
        command_executor: CommandExecutionContext<'t>,
        app_events: EventLoopProxy<AppEvents>,
    ) -> Self {
        let socket = UdpSocket::bind(bind).expect("could not bind socket");
        socket
            .set_nonblocking(true)
            .expect("could not enter non blocking mode");

        Self {
            socket,
            stop_rx,
            command_executor,
            app_events,
            buf: [0; BUF_SIZE],
        }
    }

    pub(crate) fn run(&mut self) {
        while self.stop_rx.try_recv().is_err() {
            if let Some(cmd) = self.receive_into_buf().and_then(|amount| {
                Self::command_from_slice(&self.buf[..amount])
            }) {
                debug!("received {cmd:?}");
                match cmd.execute(&self.command_executor) {
                    ExecutionResult::Success => {
                        self.app_events
                            .send_event(AppEvents::UdpPacketHandled)
                            .expect("could not send packet handled event");
                    }
                    ExecutionResult::Failure => {
                        error!("failed to execute command");
                    }
                    ExecutionResult::Shutdown => {
                        self.app_events
                            .send_event(AppEvents::UdpThreadClosed)
                            .expect("could not send close event");
                        break;
                    }
                }
            }
        }
    }

    fn command_from_slice(slice: &[u8]) -> Option<TypedCommand> {
        let packet = servicepoint::Packet::try_from(slice)
            .inspect_err(|_| {
                warn!("could not load packet with length {}", slice.len())
            })
            .ok()?;
        TypedCommand::try_from(packet)
            .inspect_err(move |err| {
                warn!("could not read command for packet: {:?}", err)
            })
            .ok()
    }

    fn receive_into_buf(&mut self) -> Option<usize> {
        let (amount, _) = match self.socket.recv_from(&mut self.buf) {
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(1));
                return None;
            }
            Ok(result) => result,
            other => other.unwrap(),
        };

        if amount == self.buf.len() {
            warn!(
                "the received package may have been truncated to a length of {}",
                amount
            );
        }
        Some(amount)
    }
}
