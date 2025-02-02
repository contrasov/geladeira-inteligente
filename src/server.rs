use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct SystemState {
    // Sensores
    pub temperatura_interna: f32,
    pub nivel_estoque: u8, // 0-100%
    pub porta_aberta: bool,
    pub ultima_atualizacao_porta: Option<Instant>,
    
    // Atuadores
    pub refrigerador_ligado: bool,
    pub luz_ligada: bool,
    pub alarme_ativado: bool,
    
    // Configurações
    pub temperatura_ideal: f32,
    pub tempo_alerta_porta: u64, // segundos
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            temperatura_interna: 4.0,
            nivel_estoque: 0,
            porta_aberta: false,
            ultima_atualizacao_porta: None,
            refrigerador_ligado: false,
            luz_ligada: false,
            alarme_ativado: false,
            temperatura_ideal: 4.0,
            tempo_alerta_porta: 30,
        }
    }
}

/* criando nosso socket vinculado ao endereço 127.0.0.1:8080*/
pub async fn start_server() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("🚀 Servidor rodando em 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("📡 Nova conexão de: {:?}", addr);

        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            match socket.read(&mut buffer).await {
                Ok(size) if size > 0 => {
                    let request = String::from_utf8_lossy(&buffer[..size]);
                    println!("📥 Mensagem recebida: {}", request);

                    // Responde ao cliente
                    let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, client!";
                    socket.write_all(response.as_bytes()).await.unwrap();
                }
                _ => println!("⚠️ Conexão encerrada."),
            }
        });
    }
}

pub async fn control_loop(state: Arc<Mutex<SystemState>>) {
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    
    loop {
        interval.tick().await;
        let mut state = state.lock().await;

        // Controle da luz e alarme da porta
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

        // Controle do refrigerador
        state.refrigerador_ligado = state.temperatura_interna > state.temperatura_ideal;

        println!(
            "❄️ Status: Temp {:.1}°C | Porta: {} | Luz: {} | Refrigerador: {} | Alarme: {}",
            state.temperatura_interna,
            if state.porta_aberta { "ABERTA" } else { "FECHADA" },
            if state.luz_ligada { "ACESA" } else { "APAGADA" },
            if state.refrigerador_ligado { "LIGADO" } else { "DESLIGADO" },
            if state.alarme_ativado { "🚨 ATIVADO" } else { "desativado" }
        );
    }
}