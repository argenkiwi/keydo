mod keys;
mod config;
mod ini;
mod macro_parser;
mod config_parser;
mod device;
mod keyboard;
mod vkbd;
mod unicode;
mod ipc;
mod tests;

use clap::{Parser, Subcommand};
use crate::config_parser::ConfigParser;
use crate::device::scan_devices;
use crate::keyboard::Keyboard;
use crate::vkbd::Vkbd;
use crate::ipc::{create_server, IpcMessage, IpcMessageType};
use nix::poll::{poll, PollFd, PollFlags};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::io::Read;

// ... (Cli and Commands structs unchanged)

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Monitor {
        #[arg(short, long)]
        time: bool,
    },
    Reload,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Monitor { time }) => {
            run_monitor(time)?;
        }
        Some(Commands::Reload) => {
            // TODO: IPC call to reload
        }
        None => {
            run_daemon()?;
        }
    }

    Ok(())
}

use std::os::unix::io::BorrowedFd;

// ...

fn run_monitor(show_time: bool) -> anyhow::Result<()> {
    let mut devices = scan_devices();
    let borrowed_fds: Vec<BorrowedFd> = devices.iter().map(|d| unsafe { BorrowedFd::borrow_raw(d.device.as_raw_fd()) }).collect();
    let mut poll_fds: Vec<PollFd> = borrowed_fds.iter().map(|fd| PollFd::new(fd, PollFlags::POLLIN)).collect();

    println!("Monitoring {} devices...", devices.len());

    let mut last_time = std::time::Instant::now();

    loop {
        poll(&mut poll_fds, -1)?;

        for i in 0..poll_fds.len() {
            if let Some(revents) = poll_fds[i].revents() {
                if revents.contains(PollFlags::POLLIN) {
                    let events: Vec<_> = devices[i].device.fetch_events()?.collect();
                    for ev in events {
                        if ev.event_type() == evdev::EventType::KEY {
                            if show_time {
                                let now = std::time::Instant::now();
                                print!("+{} ms\t", now.duration_since(last_time).as_millis());
                                last_time = now;
                            }
                            println!("{}\t{}\t{}\t{}", 
                                devices[i].device.name().unwrap_or("Unknown"),
                                devices[i].id,
                                crate::keys::get_key_name(ev.code()),
                                if ev.value() == 1 { "down" } else { "up" }
                            );
                        }
                    }
                }
            }
        }
    }
}

fn run_daemon() -> anyhow::Result<()> {
    let config_dir = Path::new("/etc/keyd"); 
    let mut parser = ConfigParser::new();
    
    let config = if let Ok(c) = parser.parse(&config_dir.join("default.conf")) {
        c
    } else {
        let mut c = parser.parse(&Path::new("/dev/null")).unwrap();
        c.wildcard = true;
        c
    };

    let mut keyboard = Keyboard::new(config);
    let mut vkbd = Vkbd::new("keydo virtual keyboard")?;
    
    let mut grabbed_devices = Vec::new();
    for mut dev in scan_devices() {
        if keyboard.config.wildcard {
             if let Ok(_) = dev.grab() {
                 grabbed_devices.push(dev);
             }
        }
    }

    let ipc_listener = create_server()?;
    let borrowed_dev_fds: Vec<BorrowedFd> = grabbed_devices.iter().map(|d| unsafe { BorrowedFd::borrow_raw(d.device.as_raw_fd()) }).collect();
    let mut poll_fds: Vec<PollFd> = borrowed_dev_fds.iter().map(|fd| PollFd::new(fd, PollFlags::POLLIN)).collect();
    let listener_fd = unsafe { BorrowedFd::borrow_raw(ipc_listener.as_raw_fd()) };
    poll_fds.push(PollFd::new(&listener_fd, PollFlags::POLLIN));

    loop {
        poll(&mut poll_fds, -1)?;

        // Handle IPC
        if let Some(revents) = poll_fds.last().unwrap().revents() {
            if revents.contains(PollFlags::POLLIN) {
                if let Ok((mut stream, _)) = ipc_listener.accept() {
                    let mut buf = Vec::new();
                    if let Ok(_) = stream.read_to_end(&mut buf) {
                        if let Ok(msg) = serde_json::from_slice::<IpcMessage>(&buf) {
                            match msg.message_type {
                                IpcMessageType::Reload => {
                                    // TODO: Implement reload logic
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Handle Devices
        for i in 0..grabbed_devices.len() {
            if let Some(revents) = poll_fds[i].revents() {
                if revents.contains(PollFlags::POLLIN) {
                    let events: Vec<_> = grabbed_devices[i].device.fetch_events()?.collect();
                    for ev in events {
                        if ev.event_type() == evdev::EventType::KEY {
                            let output_events = keyboard.process_event(ev.code(), ev.value() != 0);
                            for out_ev in output_events {
                                vkbd.send_key(out_ev.code, out_ev.pressed)?;
                            }
                        }
                    }
                }
            }
        }
    }
}
