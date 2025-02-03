use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::{Mutex};
use std::time::{Duration, Instant};
use crate::tratamento::{handle_client_command};
use uuid::Uuid;
use tokio::sync::mpsc::Sender;

#[derive(Default)]
pub struct EstadoSistema {
    // Sensores
    pub temperatura_interna: f32,
    pub nivel_estoque: u8, 
    pub porta_aberta: bool,
    pub ultima_atualizacao_porta: Option<Instant>,
    pub id_temperatura: String,
    pub id_porta: String,
    pub id_estoque: String,
    
    // Atuadores
    pub refrigerador_ligado: bool,
    pub luz_ligada: bool,
    pub alarme_ativado: bool,
    pub id_refrigerador: String,
    pub id_luz: String,
    
    // Configurações
    pub temperatura_ideal: f32,
    pub tempo_alerta_porta: u64, 
}

impl EstadoSistema {
    pub fn novo() -> Self {
        Self {
            temperatura_interna: 4.0,
            nivel_estoque: 0,
            porta_aberta: false,
            ultima_atualizacao_porta: None,
            id_temperatura: Uuid::new_v4().to_string(), 
            id_porta: Uuid::new_v4().to_string(),
            id_estoque: Uuid::new_v4().to_string(),
            id_refrigerador: Uuid::new_v4().to_string(),
            id_luz: Uuid::new_v4().to_string(),
            refrigerador_ligado: false,
            luz_ligada: false,
            alarme_ativado: false,
            temperatura_ideal: 4.0,
            tempo_alerta_porta: 30,
        }
    }
}

pub async fn iniciar_servidor(state: Arc<Mutex<EstadoSistema>>) -> tokio::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("\r\n\r\n🚀 Servidor rodando em 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("📡 Nova conexão de: {:?}", addr);
        
        let state_clone = Arc::clone(&state);

        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            
            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => break, 
                    Ok(size) => {
                        let request = String::from_utf8_lossy(&buffer[..size]);
        
                        let response = process_request(&request, &state_clone).await;
                        
                        if let Err(e) = socket.write_all(response.as_bytes()).await {
                            println!("⚠️ Erro ao enviar resposta: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Erro na conexão: {}", e);
                        break;
                    }
                }
                buffer = [0; 1024]; 
            }
            println!("🔌 Conexão encerrada com: {:?}", addr);
        });
    }
}

pub async fn process_request(request: &str, state: &Arc<Mutex<EstadoSistema>>) -> String {
    let parts: Vec<&str> = request.split_whitespace().collect();

    if parts.first() == Some(&"SENSOR/1.0") && parts.contains(&"IDENTIFY") {
        return "GERENCIADOR/1.0 200 OK\r\n\r\n".to_string();
    }

    if parts.first() == Some(&"SENSOR/1.0") {
        return "GERENCIADOR/1.0 403 COMANDO AUTOMATICO\r\n\r\n".to_string();
    }
    
    if parts.first() == Some(&"ACTUADOR/1.0") {
        return "GERENCIADOR/1.0 403 COMANDO AUTOMATICO\r\n\r\n".to_string();
    }
    
    match parts.first() {
        Some(&"CLIENT/1.0") => handle_client_command(&parts, state).await,
        _ => "GERENCIADOR/1.0 400 ERROR\r\n\r\n".to_string()
    }
}

pub async fn loop_controle(state: Arc<Mutex<EstadoSistema>>, atuador_tx: Sender<(String, String)>) {
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    
    loop {
        interval.tick().await;
        let mut state = state.lock().await;

        if state.porta_aberta {
            if !state.luz_ligada {
                state.luz_ligada = true;
                let _ = atuador_tx.send((state.id_luz.clone(), "ACENDER".to_string())).await;
            }

            if let Some(start_time) = state.ultima_atualizacao_porta {
                if start_time.elapsed().as_secs() > state.tempo_alerta_porta {
                    if !state.alarme_ativado {
                        state.alarme_ativado = true;
                        println!("🚨 Alarme ativado! Porta aberta por mais de {} segundos", state.tempo_alerta_porta);
                        let _ = atuador_tx.send(("ALARME".to_string(), "ATIVAR".to_string())).await;
                    }
                }
            } else {
                state.ultima_atualizacao_porta = Some(Instant::now());
            }
        } else {
            if state.luz_ligada {
                state.luz_ligada = false;
                let _ = atuador_tx.send((state.id_luz.clone(), "APAGAR".to_string())).await;
            }
            state.alarme_ativado = false;
            state.ultima_atualizacao_porta = None;
        }

        let _refrigerador_anterior = state.refrigerador_ligado;
        if state.temperatura_interna > state.temperatura_ideal {
            if !state.refrigerador_ligado {
                println!("❄️ Refrigerador LIGADO 🔌 (temperatura acima do limite)");
                state.refrigerador_ligado = true;
                let _ = atuador_tx.send((state.id_refrigerador.clone(), "LIGAR".to_string())).await;
            }
        } else {
            if state.refrigerador_ligado {
                println!("❄️ Refrigerador DESLIGADO 🔋 (temperatura dentro do limite)");
                state.refrigerador_ligado = false;
                let _ = atuador_tx.send((state.id_refrigerador.clone(), "DESLIGAR".to_string())).await;
            }
        }

        if state.refrigerador_ligado {
            let nova_temp = state.temperatura_interna - 0.5;
            state.temperatura_interna = nova_temp.clamp(state.temperatura_ideal - 5.0, 50.0);
            println!("🌡️ Resfriando ambiente: {:.1}°C  {:.1}°C", nova_temp + 0.5, state.temperatura_interna);
        }
    }
}