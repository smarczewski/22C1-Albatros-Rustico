#[cfg(test)]
mod tests {
    use crate::errors::MessageError;
    use crate::p2p_messages::message_builder::MessageBuilder;
    use crate::p2p_messages::message_builder::P2PMessage;
    use crate::p2p_messages::message_trait::Message;

    use crate::p2p_messages::bitfield::BitfieldMessage;
    use crate::p2p_messages::cancel::CancelMessage;
    use crate::p2p_messages::choke::ChokeMessage;
    use crate::p2p_messages::have::HaveMessage;
    use crate::p2p_messages::interested::InterestedMessage;
    use crate::p2p_messages::keep_alive::KeepAliveMessage;
    use crate::p2p_messages::not_interested::NotInterestedMessage;
    use crate::p2p_messages::piece::PieceMessage;
    use crate::p2p_messages::request::RequestMessage;
    use crate::p2p_messages::unchoke::UnchokeMessage;

    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc;
    use std::sync::mpsc::Sender;
    use std::thread;

    fn receive_msgs(
        stream: &mut TcpStream,
        number_of_msg: u32,
    ) -> Result<Vec<P2PMessage>, MessageError> {
        let mut vec = Vec::new();
        for _i in 0..number_of_msg {
            let msg = MessageBuilder::build(stream)?;
            vec.push(msg);
        }
        Ok(vec)
    }

    fn handle_client(tx_cl: Sender<Vec<P2PMessage>>, address: &str) -> Result<(), MessageError> {
        let mut stream_cl = TcpStream::connect(address).unwrap();

        let cl_msgs = receive_msgs(&mut stream_cl, 6)?;
        tx_cl.send(cl_msgs).unwrap();

        let interested_msg = InterestedMessage::new();
        interested_msg.send_msg(&mut stream_cl)?;

        let notinterested_msg = NotInterestedMessage::new();
        notinterested_msg.send_msg(&mut stream_cl)?;

        let request_msg = RequestMessage::new(5, 0, 1024)?;
        request_msg.send_msg(&mut stream_cl)?;

        let cancel_msg = CancelMessage::new(5, 0, 1024);
        cancel_msg.send_msg(&mut stream_cl)?;

        Ok(())
    }

    fn handle_server(
        tx_sv: Sender<Vec<P2PMessage>>,
        listener: TcpListener,
    ) -> Result<(), MessageError> {
        let (mut stream_sv, _socket_addr) = listener.accept().unwrap();

        let bitfield_msg = BitfieldMessage::new(vec![1, 0, 2, 99])?;
        bitfield_msg.send_msg(&mut stream_sv)?;

        let have_msg = HaveMessage::new(10);
        have_msg.send_msg(&mut stream_sv)?;

        let keep_alive_msg = KeepAliveMessage::new();
        keep_alive_msg.send_msg(&mut stream_sv)?;

        let choke_msg = ChokeMessage::new();
        choke_msg.send_msg(&mut stream_sv)?;

        let unchoke_msg = UnchokeMessage::new();
        unchoke_msg.send_msg(&mut stream_sv)?;

        let piece_msg = PieceMessage::new(5, 0, vec![10, 16, 255])?;
        piece_msg.send_msg(&mut stream_sv)?;

        let sv_msgs = receive_msgs(&mut stream_sv, 4)?;
        tx_sv.send(sv_msgs).unwrap();

        Ok(())
    }

    fn setup() -> (Vec<P2PMessage>, Vec<P2PMessage>) {
        let address = "127.0.0.1:8080";
        let listener = TcpListener::bind(address).unwrap();

        let (tx_cl, rx_cl) = mpsc::channel();
        let client_thread = thread::spawn(move || {
            handle_client(tx_cl, address).unwrap();
        });

        let (tx_sv, rx_sv) = mpsc::channel();
        let server_thread = thread::spawn(move || {
            handle_server(tx_sv, listener).unwrap();
        });

        client_thread.join().unwrap();
        server_thread.join().unwrap();

        let received_cl = rx_cl.recv().unwrap();
        let received_sv = rx_sv.recv().unwrap();

        (received_cl, received_sv)
    }

    #[test]
    fn msgs_betw_client_server() {
        let (mut received_cl, mut received_sv) = setup();

        let unchoke_expected = UnchokeMessage::new();
        let choke_expected = ChokeMessage::new();
        let keep_alive_expected = KeepAliveMessage::new();
        let have_expected = HaveMessage::new(10);
        let interested_expected = InterestedMessage::new();
        let not_interested_expected = NotInterestedMessage::new();
        let cancel_expected = CancelMessage::new(5, 0, 1024);
        let request_expected = RequestMessage::new(5, 0, 1024).expect("error message creation");
        let piece_expected =
            PieceMessage::new(5, 0, vec![10, 16, 255]).expect("error message creation");
        let bitfield_expected =
            BitfieldMessage::new(vec![1, 0, 2, 99]).expect("error message creation");

        match received_cl.pop() {
            Some(P2PMessage::Piece(m)) => assert_eq!(m, piece_expected),
            _ => assert!(false),
        }

        match received_cl.pop() {
            Some(P2PMessage::Unchoke(m)) => assert_eq!(m, unchoke_expected),
            _ => assert!(false),
        }

        match received_cl.pop() {
            Some(P2PMessage::Choke(m)) => assert_eq!(m, choke_expected),
            _ => assert!(false),
        }

        match received_cl.pop() {
            Some(P2PMessage::KeepAlive(m)) => assert_eq!(m, keep_alive_expected),
            _ => assert!(false),
        }

        match received_cl.pop() {
            Some(P2PMessage::Have(m)) => assert_eq!(m, have_expected),
            _ => assert!(false),
        }

        match received_cl.pop() {
            Some(P2PMessage::Bitfield(m)) => assert_eq!(m, bitfield_expected),
            _ => assert!(false),
        }

        match received_sv.pop() {
            Some(P2PMessage::Cancel(m)) => assert_eq!(m, cancel_expected),
            _ => assert!(false),
        }

        match received_sv.pop() {
            Some(P2PMessage::Request(m)) => assert_eq!(m, request_expected),
            _ => assert!(false),
        }

        match received_sv.pop() {
            Some(P2PMessage::NotInterested(m)) => assert_eq!(m, not_interested_expected),
            _ => assert!(false),
        }

        match received_sv.pop() {
            Some(P2PMessage::Interested(m)) => assert_eq!(m, interested_expected),
            _ => assert!(false),
        }
    }
}
