use std::io::prelude::*;
use std::net::TcpStream;
pub struct Cliente {
    pub vector_conexiones: Vec<TcpStream>,
}

impl Cliente {
    pub fn connect(&mut self, puerto: String) -> Result<String, String> {
        let socket_conectado = TcpStream::connect(puerto);
        match socket_conectado {
            Ok(socket_conectado) => {
                self.vector_conexiones.push(socket_conectado);
                Ok("Conexion establecida".to_string())
            }
            Err(_e) => Err("No se pudo realizar la conexion".to_string()),
        }
    }

    pub fn send(&mut self, buffer: &[u8]) -> Result<String, String> {
        let mut conexion = self.vector_conexiones.pop().unwrap();
        match conexion.write(buffer) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    //se requiere de la creacion de un servidor en el puerto especificado
    fn se_establece_una_conexion_correctamente_con_un_servidor() {
        let mut cliente = Cliente {
            vector_conexiones: Vec::new(),
        };
        let puerto = "127.0.0.1:34255".to_string();
        assert!(Result::is_ok(&cliente.connect(puerto)));
    }
}
