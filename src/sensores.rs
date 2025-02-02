use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use crate::servidor::EstadoSistema;

#[derive(Debug, Clone)]
pub enum TipoSensor {
    Temperatura,
    Estoque,
    Porta,
}

#[derive(Debug)]
pub struct Sensor {
    pub id: Uuid,
    pub tipo: TipoSensor,
}

impl Sensor {
    pub fn novo(tipo: TipoSensor) -> Self {
        Self {
            id: Uuid::new_v4(),
            tipo,
        }
    }

    pub async fn iniciar(self, estado: Arc<Mutex<EstadoSistema>>, transmissor: mpsc::Sender<String>) {
        loop {
            sleep(Duration::from_secs(10)).await;

            let leitura = match self.tipo {
                TipoSensor::Temperatura => {
                    let mut estado = estado.lock().await;
                    estado.temperatura_interna += rand::random::<f32>() * 2.0 - 1.0; // Simula variação de temperatura
                    format!(
                        "SENSOR/1.0\r\nTEMPERATURA {:.1}\r\nID {}\r\n\r\n",
                        estado.temperatura_interna, self.id
                    )
                }
                TipoSensor::Estoque => {
                    let mut estado = estado.lock().await;
                    estado.nivel_estoque = (rand::random::<u8>() % 100) + 1; // Simula mudança no estoque
                    format!(
                        "SENSOR/1.0\r\nESTOQUE {}\r\nID {}\r\n\r\n",
                        estado.nivel_estoque, self.id
                    )
                }
                TipoSensor::Porta => {
                    let status = rand::random::<bool>();
                    format!(
                        "SENSOR/1.0 PORTA {} ID {}\r\n\r\n",
                        if status { "ABERTA" } else { "FECHADA" },
                        self.id
                    )
                }
            };

            if let Err(_) = transmissor.send(leitura).await {
                println!("⚠️ Falha ao enviar leitura do sensor {}", self.id);
            }
        }
    }
}
