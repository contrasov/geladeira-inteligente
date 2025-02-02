use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
mod tratamento;
mod servidor;
mod sensores;
use crate::servidor::{EstadoSistema, loop_controle, iniciar_servidor};
use crate::sensores::{Sensor, TipoSensor};

#[tokio::main]
async fn main() {
    let estado = Arc::new(Mutex::new(EstadoSistema::novo()));

    // Criando um canal de comunicação entre sensores e o servidor
    let (transmissor, mut receptor) = mpsc::channel::<String>(10);

    // Inicializa sensores
    let sensores = vec![
        Sensor::novo(TipoSensor::Temperatura),
        Sensor::novo(TipoSensor::Estoque),
        Sensor::novo(TipoSensor::Porta),
    ];

    for sensor in sensores {
        let estado_clone = Arc::clone(&estado);
        let transmissor_clone = transmissor.clone();
        tokio::spawn(async move {
            sensor.iniciar(estado_clone, transmissor_clone).await;
        });
    }

    let estado_clone = Arc::clone(&estado);
    tokio::spawn(async move {
        while let Some(leitura) = receptor.recv().await {
            let estado_clone = Arc::clone(&estado_clone);
            tokio::spawn(async move {
                let _ = tratamento::tratar_dados_sensor(&[&leitura], &estado_clone).await;
            });
        }
    });

    // Inicia o loop de controle
    let estado_clone = Arc::clone(&estado);
    tokio::spawn(async move {
        loop_controle(estado_clone).await;
    });

    // Inicia o servidor
    if let Err(e) = iniciar_servidor(estado).await {
        eprintln!("Erro ao iniciar o servidor: {}", e);
    }
}
