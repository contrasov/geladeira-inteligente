use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use crate::tratamento::{handle_client_command, tratar_dados_sensor, handle_actuator_command};

#[derive(Default)]
pub struct EstadoSistema {
    // Sensores
    pub temperatura_interna: f32,
    pub nivel_estoque: u8, // 0-100%
    pub porta_aberta: bool,
    pub ultima_atualizacao_porta: Option<Instant>,
    pub id_temperatura: String,
    pub id_porta: String,
    pub id_estoque: String,
    
    // Atuadores
    pub refrigerador_ligado: bool,
    pub luz_ligada: bool,
    pub alarme_ativado: bool,
    
    // Configurações
    pub temperatura_ideal: f32,
    pub tempo_alerta_porta: u64, // segundos
}

impl EstadoSistema {
    pub fn novo() -> Self {
        Self {
            temperatura_interna: 4.0,
            nivel_estoque: 0,
            porta_aberta: false,
            ultima_atualizacao_porta: None,
            id_temperatura: String::new(),
            id_porta: String::new(),
            id_estoque: String::new(),
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
    println!("🚀 Servidor rodando em 127.0.0.1:8080");

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

        // Controle da luz e  da porta
        if state.porta_aberta {
            state.luz_ligada = true;
            
            if let Some(start_time) = state.ultima_atualizacao_porta {
                if start_time.elapsed().as_secs() > state.tempo_alerta_porta {
                    state.alarme_ativado = true;
                }
            } else {
                state.ultima_atualizacao_porta = Some(Instant::now());
            }
        } else {
            state.luz_ligada = false;
            state.alarme_ativado = false;
            state.ultima_atualizacao_porta = None;
        }

        state.refrigerador_ligado = state.temperatura_interna > state.temperatura_ideal;

    }
}