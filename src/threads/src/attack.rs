use crate::ipc::{IPC,IPCMessage};
use wpa::Handshake;

//A worker for cracking the network's password
pub fn password_worker(ipc: IPC<String>,handshake: Handshake){
    loop {
        if let Ok(ipc_msg) = ipc.rx.recv(){
            match ipc_msg{
                IPCMessage::Message(password) =>{
                    println!("[-] trying: {}",&password);
                    if handshake.clone().try_password(&password){
                        ipc.tx.send(IPCMessage::Message(password));
                        return;
                    }else{
                        ipc.tx.send(IPCMessage::Message("wrong".to_owned()));
                    }
                },
                IPCMessage::EndCommunication => {
                    return;
                },
                _ => {
                    continue;
                }
            }
        }
    }
}
