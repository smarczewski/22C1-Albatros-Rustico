use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

pub struct Servidor {
    pub vector_listener: Vec<TcpListener>,
    pub vector_conexiones: Vec<TcpStream>,
}

impl Servidor {
    pub fn bind(&mut self, puerto: String) -> Result<String, String> {
        let copia_string = puerto.clone();
        let listener = TcpListener::bind(puerto);
        let msje_error = format!("Fallo el bind en la direccion: {}", copia_string);
        let listener = match listener {
            Ok(listener) => listener,
            Err(_error) => return Err(msje_error),
        };
        self.vector_listener.push(listener);
        Ok("bind correcto".to_string())
    }

    ///esta funcion bloquea el thread
    pub fn accept(&mut self) -> Result<String, String> {
        let listener = self.vector_listener.pop().unwrap();
        match listener.accept() {
            Ok((_socket, addr)) => {
                self.vector_conexiones.push(_socket);
                println!("Nuevo cliente: {:?}", addr);
                self.vector_listener.push(listener);
                Ok("Conexion ok".to_string())
            }
            Err(_e) => {
                self.vector_listener.push(listener);
                Err("Fallo accept".to_string())
            }
        }
    }
    ///TODO: recibir en un lugar arbitrario
    pub fn receive(&mut self) -> Result<String, String> {
        if self.vector_conexiones.is_empty() {
            return Err(
                "No hay conexiones establecidas con el servidor. No se puede recibir".to_string(),
            );
        }
        let mut buffer = [0; 1024];
        let mut conexion = self.vector_conexiones.pop().unwrap();
        match conexion.read(&mut buffer) {
            Ok(_bytes_leidos) => println!("Se recibio un mensaje"),
            Err(_e) => return Err("Fallo de lectura".to_string()),
        };
        //println!("Request: {}",String::from_utf8_lossy(&buffer[..]));
        let msje_recibido = String::from_utf8_lossy(&buffer[..]);
        self.vector_conexiones.push(conexion);
        Ok(msje_recibido.to_string())
    }

    pub fn send(&mut self, _buffer: &[u8]) -> Result<String, String> {
        let mut conexion = self.vector_conexiones.pop().unwrap();
        match conexion.write(b"mensaje del servidor") {
            Ok(_bytes_escritos) => {
                self.vector_conexiones.push(conexion);
                Ok("Se envio un mensaje correctamente".to_string())
            }
            Err(_e) => {
                self.vector_conexiones.push(conexion);
                Err("Fallo la escritura de los mensajes".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creacion_de_un_servidor() {
        let _servidor = Servidor {
            vector_listener: Vec::new(),
            vector_conexiones: Vec::new(),
        };
    }

    #[test]
    fn el_servidor_bindea_correctamente_a_un_puerto_valido() {
        let mut servidor = Servidor {
            vector_listener: Vec::new(),
            vector_conexiones: Vec::new(),
        };
        let puerto = "127.0.0.1:34255".to_string();
        assert!(Result::is_ok(&servidor.bind(puerto)));
    }
    #[test]
    fn el_servidor_no_bindea_a_un_puerto_invalido() {
        let mut servidor = Servidor {
            vector_listener: Vec::new(),
            vector_conexiones: Vec::new(),
        };
        let puerto = "un_puerto".to_string();
        assert!(Result::is_err(&servidor.bind(puerto)));
    }
    //#[test]
    //fn el_servidor_acepta_correctamente_un_mensaje(){
    //	let mut servidor = Servidor{vector_listener: Vec::new(),vector_conexiones: Vec::new(),};
    //	let puerto = "127.0.0.1:34255".to_string();
    //	servidor.bind(puerto);
    //	servidor.accept();
    //	assert!(Result::is_ok(&servidor.receive()));
    //}

    //para el test de este caso se requiere conectarse por otro lado
    //ejemplo: usando un navegador web
    //#[test]
    //fn el_servidor_acepta_correctamente_una_nueva_conexion(){
    //	let mut servidor = Servidor{vector_listener: Vec::new(),vector_conexiones: Vec::new(),};
    //	let puerto = "127.0.0.1:34254".to_string();
    //	servidor.bind(puerto);
    //	assert!(Result::is_ok(&servidor.accept()));
    //}

    //#[test]
    //fn servidor_envia_msj_correctamente(){
    //	let mut servidor = Servidor{vector_listener: Vec::new(),vector_conexiones: Vec::new(),};
    //	let puerto = "127.0.0.1:34254".to_string();
    //	let mut buffer = [0; 1024];
    //	servidor.bind(puerto);
    //	servidor.accept();
    //	servidor.send(&buffer);
    //}
}
