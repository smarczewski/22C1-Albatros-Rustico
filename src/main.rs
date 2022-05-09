//use std::io::prelude::*;
//use std::net::TcpListener;
//use std::net::TcpStream;
use std::env;
mod servidor;
use servidor::*;
mod cliente;
use cliente::*;
//Este es un codigo que arme para probar el codigo de las pruebas
//para cliente-servidor. Genera un cliente o un servidor en el
//puerto especificado.
//la sintaxis es cargo run modo puerto
//donde modo puede ser "s" o "c"
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Incorrecto numero de argumentos");
        return;
    }
    let modo = &args[1];
    let puerto = &args[2];
    if modo == "s" {
        //modo servidor
        let msje_error = format!("Fallo el bind en la direccion: {}", &puerto);
        let mut servidor = Servidor {
            vector_listener: Vec::new(),
            vector_conexiones: Vec::new(),
        };
        match servidor.bind(puerto.to_string()) {
            Ok(_bindeo) => println!("Funciono el bindeo"),
            Err(_error) => {
                println!("{}", msje_error);
                return;
            }
        };
        //se acepta una sola conexion
        match servidor.accept() {
            Ok(_o) => println!("Conexion aceptada"),
            Err(_e) => {
                println!("No se acepto una conexion");
                return;
            }
        }
        match servidor.receive() {
            Ok(msje_recibido) => println!("{}", msje_recibido),
            Err(_e) => {
                println!("Fallo la recepcion de mensaje");
                return;
            }
        };
        match servidor.send(b"Mensaje respuesta del servidor") {
            Ok(_o) => {
                println!("Se envio un mensaje al cliente");
            }
            Err(_e) => {
                println!("Fallo el envio del mensaje al cliente");
            }
        }
    } else {
        //modo cliente
        let mut cliente = Cliente {
            vector_conexiones: Vec::new(),
        };

        //let sender = TcpStream::connect(puerto);
        let msje_error = format!("Fallo el connect en la direccion {}", puerto);
        match cliente.connect(puerto.to_string()) {
            Ok(_conecto) => println!("El cliente conecto correctamente con el servidor"),
            Err(_e) => println!("{}", msje_error),
        };

        match cliente.send(b"mensaje del cliente") {
            Ok(msje_enviado) => println!("{}", msje_enviado),
            Err(_e) => {
                println!("{}", _e);
                return;
            }
        };

        //codigo de testeo por si el cliente quiere recibir un mensaje
        //enviado por el servidor
        match cliente.receive() {
            Ok(msje_recibido) => println!("Mensaje recibido: {}", msje_recibido),
            Err(_e) => {
                println!("Fallo en la recepcion del mensaje");
            }
        };
    }
}
