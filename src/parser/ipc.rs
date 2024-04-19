use interprocess::local_socket::LocalSocketStream;
use std::{io::BufReader, process, sync::mpsc::RecvTimeoutError, thread, time};

use crate::{
    daemon::{discord::DiscordThreadCommands, socket::write, ChannelCommunications},
    logging::ddrpc_log,
};

pub fn ipc_parser(
    buffer: String,
    socket_stream: &mut BufReader<LocalSocketStream>,
    channel_communications: &ChannelCommunications,
) -> () {
    let split_buffer: Vec<String> = buffer
        .split_whitespace()
        .map(|str| str.to_lowercase())
        .collect();
    let final_buffer: Vec<&str> = split_buffer.iter().map(|string| string.as_ref()).collect();

    thread::sleep(time::Duration::from_millis(100));

    match final_buffer[0] {
        "ping" => write(socket_stream, b"pong"),
        "kill" => {
            write(socket_stream, b"Killing process");
            process::exit(0);
        }
        "discord" => match final_buffer[1] {
            // "get" => {
            //     channel_communications
            //         .discord
            //         .send(DiscordThreadCommands::Get)
            //         .unwrap();
            //     write(
            //         socket_stream,
            //         match &channel_communications
            //             .main
            //             .recv_timeout(time::Duration::from_secs(5))
            //         {
            //             Err(error) if error == &RecvTimeoutError::Timeout => {
            //                 b"Discord RPC thread did not respond"
            //             }
            //             Err(_error) => b"Discord RPC thread's sender disconnected", // Probably retry Discord thread initialization
            //             Ok(buffer) => buffer,
            //         },
            //     )
            // }
            "connect" => {
                match channel_communications
                    .discord
                    .send(DiscordThreadCommands::Connect)
                {
                    Ok(_) => {}
                    Err(error) => {
                        ddrpc_log(&format!("{}", error));
                        write(socket_stream, format!("{}", error).as_bytes())
                    }
                };
                ddrpc_log("connection command sent")
            }
            "disconnect" => {
                channel_communications
                    .discord
                    .send(DiscordThreadCommands::Disconnect)
                    .unwrap();
                ddrpc_log("disconnection command sent")
            }
            "update" => channel_communications
                .discord
                .send(DiscordThreadCommands::Update)
                .unwrap(),
            str => write(
                socket_stream,
                format!("Unknown discord command: {str}").as_bytes(),
            ),
        },
        _ => write(socket_stream, b"Unknown input"),
    }
}
