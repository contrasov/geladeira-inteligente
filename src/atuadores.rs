use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, oneshot};
use uuid::Uuid;
use crate::servidor::EstadoSistema;

#[derive(Debug, Clone)]
pub enum TipoAtuador {
    Refrigerador,
    Luz,
    Alarme,
}

#[derive(Debug)]
pub struct Atuador {
    pub id: Uuid,
    pub tipo: TipoAtuador,
}

impl Atuador {
    pub fn novo(tipo: TipoAtuador) -> Self {
        Self {
            id: Uuid::new_v4(),
            tipo,
        }
    }

    pub async fn iniciar(
        self,
        estado: Arc<Mutex<EstadoSistema>>,
        transmissor: mpsc::Sender<String>,
        confirmacao_rx: oneshot::Receiver<String>,
        mut comando_rx: mpsc::Receiver<String>,
    ) {
        let ident_msg = format!("ACTUADOR/1.0 IDENTIFY ID {}\r\n\r\n", self.id);
        if let Err(_) = transmissor.send(ident_msg).await {
            println!("⚠️ Atuador {}: Falha ao enviar identificação.", self.id);
            return;
        }

        match confirmacao_rx.await {
            Ok(resp) if resp.contains("200 OK") => {
                println!("Atuador {}: Confirmado.", self.id);
            }
            _ => {
                println!("Atuador {}: Falha na confirmação.", self.id);
                return;
            }
        }

        while let Some(comando) = comando_rx.recv().await {
            println!("Atuador {}: Recebeu comando: {}", self.id, comando);
            let mut estado_guard = estado.lock().await;
            match self.tipo {
                TipoAtuador::Refrigerador => {
                    if comando.to_uppercase().contains("LIGAR") {
                        estado_guard.refrigerador_ligado = true;
                    } else if comando.to_uppercase().contains("DESLIGAR") {
                        estado_guard.refrigerador_ligado = false;
                    }
                }
                TipoAtuador::Luz => {
                    if comando.to_uppercase().contains("ACENDER") {
                        estado_guard.luz_ligada = true;
                    } else if comando.to_uppercase().contains("APAGAR") {
                        estado_guard.luz_ligada = false;
                    }
                }
                TipoAtuador::Alarme => {
                    if comando.to_uppercase().contains("ATIVAR") {
                        estado_guard.alarme_ativado = true;
                    } else if comando.to_uppercase().contains("DESATIVAR") {
                        estado_guard.alarme_ativado = false;
                    }
                }
            }
            println!("🎛️ Atuador {}: Executou '{}' → Estado atualizado", self.id, comando);        }
    }
}
