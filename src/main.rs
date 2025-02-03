use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

mod servidor;
mod sensores;
mod atuadores;
mod tratamento;

use servidor::{EstadoSistema, iniciar_servidor, loop_controle};
use sensores::{Sensor, TipoSensor};
use atuadores::{Atuador, TipoAtuador};

#[tokio::main]
async fn main() {
    let estado = Arc::new(Mutex::new(EstadoSistema::novo()));
    
    let confirmacoes: Arc<Mutex<HashMap<Uuid, tokio::sync::oneshot::Sender<String>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let atuadores_tx: Arc<Mutex<HashMap<Uuid, mpsc::Sender<String>>>> = 
        Arc::new(Mutex::new(HashMap::new()));
        
    let (transmissor, mut receptor) = mpsc::channel::<String>(10);
    
    // Inicializa os sensores.
    let sensores_vec = vec![
        Sensor::novo(TipoSensor::Temperatura),
        Sensor::novo(TipoSensor::Estoque),
        Sensor::novo(TipoSensor::Porta),
    ];
    
    for sensor in sensores_vec {
        let (tx_confirm, rx_confirm) = tokio::sync::oneshot::channel::<String>();
        {
            let mut map = confirmacoes.lock().await;
            map.insert(sensor.id, tx_confirm);
        }
        let estado_clone = Arc::clone(&estado);
        let transmissor_clone = transmissor.clone();
        tokio::spawn(async move {
            sensor.iniciar(estado_clone, transmissor_clone, rx_confirm).await;
        });
    }
    
    let atuadores_vec = vec![
        Atuador::novo(TipoAtuador::Refrigerador),
        Atuador::novo(TipoAtuador::Luz),
        Atuador::novo(TipoAtuador::Alarme),
    ];
    
    for atuador in atuadores_vec {
        let (tx_confirm, rx_confirm) = tokio::sync::oneshot::channel::<String>();
        {
            let mut map = confirmacoes.lock().await;
            map.insert(atuador.id, tx_confirm);
        }
        let (tx_comando, rx_comando) = mpsc::channel::<String>(5);
        
        {
            let mut map = atuadores_tx.lock().await;
            map.insert(atuador.id, tx_comando);
        }
    }
    
    let confirmacoes_clone = Arc::clone(&confirmacoes);
    tokio::spawn(async move {
        while let Some(msg) = receptor.recv().await {
            if msg.contains("IDENTIFY") {
                let parts: Vec<&str> = msg.split_whitespace().collect();
                if let Some(id_index) = parts.iter().position(|&p| p == "ID") {
                    if let Some(id_str) = parts.get(id_index + 1) {
                        if let Ok(uuid) = Uuid::parse_str(id_str) {
                            let resposta = "GERENCIADOR/1.0 200 OK\r\n\r\n".to_string();
                            let mut map = confirmacoes_clone.lock().await;
                            if let Some(tx) = map.remove(&uuid) {
                                let _ = tx.send(resposta);
                            }
                        }
                    }
                }
            }
        }
    });
    
    let (atuador_tx, mut atuador_rx) = mpsc::channel::<(String, String)>(10);

    let estado_clone = Arc::clone(&estado);
    tokio::spawn(async move {
        servidor::loop_controle(estado_clone, atuador_tx).await;
    });

    let atuadores_tx_clone = Arc::clone(&atuadores_tx);
    tokio::spawn(async move {
        while let Some((id, cmd)) = atuador_rx.recv().await {
            println!("🔀 Roteando comando: {} → {}", id, cmd);
            let mapa = atuadores_tx_clone.lock().await;
            if let Ok(uuid) = Uuid::parse_str(&id) {
                if let Some(tx) = mapa.get(&uuid) {
                    let _ = tx.send(cmd).await;
                }
            }
        }
    });
    
    if let Err(e) = iniciar_servidor(Arc::clone(&estado)).await {
        eprintln!("Erro ao iniciar o servidor: {}", e);
    }
}
