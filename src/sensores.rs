use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use crate::servidor::EstadoSistema;
use rand::Rng;

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

    pub async fn iniciar(
        self,
        estado: Arc<Mutex<EstadoSistema>>,
        transmissor: mpsc::Sender<String>,
        confirmacao_rx: oneshot::Receiver<String>
    ) {
        let ident_msg = format!("SENSOR/1.0 IDENTIFY ID {}\r\n\r\n", self.id);
        if let Err(_) = transmissor.send(ident_msg).await {
            println!("⚠️ Sensor {}: Falha ao enviar mensagem de identificação.", self.id);
            return;
        }
        println!("🔎 Sensor {}: Mensagem de identificação enviada. Aguardando confirmação...", self.id);

        match confirmacao_rx.await {
            Ok(resp) if resp.contains("200 OK") => {
                println!("✅ Sensor {}: Confirmado pelo Gerenciador.", self.id);
            }
            _ => {
                println!("❌ Sensor {}: Falha na confirmação. Abortando envios.", self.id);
                return;
            }
        }

        loop {
            sleep(Duration::from_secs(10)).await;

            let leitura = match self.tipo {
                TipoSensor::Temperatura => {
                    let mut estado = estado.lock().await;
                    let mut rng = rand::thread_rng();
                    estado.temperatura_interna += rng.gen_range(-1.0..=1.0); // Simula variação de temperatura
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
                    let estado = estado.lock().await;
                    format!(
                        "SENSOR/1.0 PORTA {} ID {}\r\n\r\n",
                        if estado.porta_aberta { "ABERTA" } else { "FECHADA" },
                        self.id
                    )
                }
            };

            if let Err(_) = transmissor.send(leitura).await {
                println!("⚠️ Sensor {}: Falha ao enviar leitura.", self.id);
            }
        }
    }
}
