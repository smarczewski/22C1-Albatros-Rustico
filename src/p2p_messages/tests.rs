#[cfg(test)]
mod tests {
    use crate::errors::MessageError;
    use crate::p2p_messages::message_builder::MessageBuilder;
    use crate::p2p_messages::message_builder::P2PMessage;
    use crate::p2p_messages::message_trait::Message;

    use crate::p2p_messages::bitfield::BitfieldMsg;
    use crate::p2p_messages::cancel::CancelMsg;
    use crate::p2p_messages::choke::ChokeMsg;
    use crate::p2p_messages::have::HaveMsg;
    use crate::p2p_messages::interested::InterestedMsg;
    use crate::p2p_messages::keep_alive::KeepAliveMsg;
    use crate::p2p_messages::not_interested::NotInterestedMsg;
    use crate::p2p_messages::piece::PieceMsg;
    use crate::p2p_messages::request::RequestMsg;
    use crate::p2p_messages::unchoke::UnchokeMsg;

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
        if let Ok(mut stream_cl) = TcpStream::connect(address) {
            let cl_msgs = receive_msgs(&mut stream_cl, 6)?;
            let _ = tx_cl.send(cl_msgs);

            let interested_msg = InterestedMsg::new();
            interested_msg.send_msg(&mut stream_cl)?;

            let notinterested_msg = NotInterestedMsg::new();
            notinterested_msg.send_msg(&mut stream_cl)?;

            let request_msg = RequestMsg::new(5, 0, 1024)?;
            request_msg.send_msg(&mut stream_cl)?;

            let cancel_msg = CancelMsg::new(5, 0, 1024);
            cancel_msg.send_msg(&mut stream_cl)?;
        }
        Ok(())
    }

    fn handle_server(
        tx_sv: Sender<Vec<P2PMessage>>,
        listener: TcpListener,
    ) -> Result<(), MessageError> {
        let (mut stream_sv, _socket_addr) = listener.accept().unwrap();

        let bitfield_msg = BitfieldMsg::new(vec![1, 0, 2, 99])?;
        bitfield_msg.send_msg(&mut stream_sv)?;

        let have_msg = HaveMsg::new(10);
        have_msg.send_msg(&mut stream_sv)?;

        let keep_alive_msg = KeepAliveMsg::new();
        keep_alive_msg.send_msg(&mut stream_sv)?;

        let choke_msg = ChokeMsg::new();
        choke_msg.send_msg(&mut stream_sv)?;

        let unchoke_msg = UnchokeMsg::new();
        unchoke_msg.send_msg(&mut stream_sv)?;

        let piece_msg = PieceMsg::new(5, 0, vec![10, 16, 255])?;
        piece_msg.send_msg(&mut stream_sv)?;

        let sv_msgs = receive_msgs(&mut stream_sv, 4)?;
        let _ = tx_sv.send(sv_msgs);

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

        let _ = client_thread.join();
        let _ = server_thread.join();

        let received_cl = rx_cl.recv().unwrap();
        let received_sv = rx_sv.recv().unwrap();

        (received_cl, received_sv)
    }

    #[test]
    fn msgs_betw_client_server() {
        let (mut received_cl, mut received_sv) = setup();

        let unchoke_expected = UnchokeMsg::new();
        let choke_expected = ChokeMsg::new();
        let keep_alive_expected = KeepAliveMsg::new();
        let have_expected = HaveMsg::new(10);
        let interested_expected = InterestedMsg::new();
        let not_interested_expected = NotInterestedMsg::new();
        let cancel_expected = CancelMsg::new(5, 0, 1024);
        let request_expected = RequestMsg::new(5, 0, 1024).expect("error message creation");
        let piece_expected =
            PieceMsg::new(5, 0, vec![10, 16, 255]).expect("error message creation");
        let bitfield_expected =
            BitfieldMsg::new(vec![1, 0, 2, 99]).expect("error message creation");

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
