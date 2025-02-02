use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::{Mutex};
use std::time::{Duration, Instant};
use crate::tratamento::{handle_client_command, tratar_dados_sensor, handle_actuator_command};
use uuid::Uuid;

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
        
        // Clona o estado para cada conexão
        let state_clone = Arc::clone(&state);

        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            
            // Mantém a conexão aberta
            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => break, // Conexão fechada pelo cliente
                    Ok(size) => {
                        let request = String::from_utf8_lossy(&buffer[..size]);
                        println!("📥 Mensagem recebida: {}", request);
        
                        // Processa a requisição
                        let response = process_request(&request, &state_clone).await;
                        
                        // Envia resposta e mantém conexão
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
                buffer = [0; 1024]; // Limpa o buffer para próxima mensagem
            }
            println!("🔌 Conexão encerrada com: {:?}", addr);
        });
    }
}

async fn process_request(request: &str, state: &Arc<Mutex<EstadoSistema>>) -> String {
    let parts: Vec<&str> = request.split_whitespace().collect();
    
    match parts.first() {
        Some(&"CLIENT/1.0") => handle_client_command(&parts, state).await,
        Some(&"SENSOR/1.0") => tratar_dados_sensor(&parts, state).await,
        Some(&"ACTUATOR/1.0") => handle_actuator_command(&parts, state).await,
        _ => "MANAGER/1.0 400 ERROR\r\n\r\n".to_string()
    }
}

pub async fn loop_controle(state: Arc<Mutex<EstadoSistema>>) {
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    
    loop {
        interval.tick().await;
        let mut state = state.lock().await;

        // Controle da luz e da porta
        if state.porta_aberta {
            state.luz_ligada = true;
            
            if let Some(start_time) = state.ultima_atualizacao_porta {
                if start_time.elapsed().as_secs() > state.tempo_alerta_porta {
                    state.alarme_ativado = true;
                    println!("🚨 Alarme ativado! Porta aberta por mais de {} segundos", state.tempo_alerta_porta);
                }
            } else {
                state.ultima_atualizacao_porta = Some(Instant::now());
            }
        } else {
            state.luz_ligada = false;
            state.alarme_ativado = false;
            state.ultima_atualizacao_porta = None;
        }

        let luz_anterior = state.luz_ligada;
        if luz_anterior != state.luz_ligada {
            println!("💡 Luz {} → Porta {}", 
                if state.luz_ligada { "ACESSA" } else { "APAGADA" },
                if state.porta_aberta { "ABERTA" } else { "FECHADA" }
            );
        }

        // Controle do refrigerador com logging
        let refrigerador_anterior = state.refrigerador_ligado;
        state.refrigerador_ligado = state.temperatura_interna > state.temperatura_ideal;

        // Log de mudança de estado
        if refrigerador_anterior != state.refrigerador_ligado {
            let status = if state.refrigerador_ligado {
                "LIGADO 🔌 (temperatura acima do limite)"
            } else {
                "DESLIGADO 🔋 (temperatura dentro do limite)"
            };
            println!(
                "❄️  Refrigerador {} → {:.1}°C | Limite: {:.1}°C",
                status, state.temperatura_interna, state.temperatura_ideal
            );
        }

        // Atualização da temperatura
        if state.refrigerador_ligado {
            let nova_temp = state.temperatura_interna - 0.5;
            state.temperatura_interna = nova_temp.clamp(
                state.temperatura_ideal - 5.0,  // Temperatura mínima
                50.0                            // Temperatura máxima de segurança
            );
            println!("🌡️  Resfriando: {:.1}°C → {:.1}°C", nova_temp + 0.5, state.temperatura_interna);
        }
    }
}