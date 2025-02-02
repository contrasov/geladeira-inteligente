use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::handlers;
use handlers::{handle_sensor_message, handle_client_message};
use std::collections::HashSet;
use std::collections::HashMap;
use rand::Rng;
use tokio::time::Duration;




#[derive(Default)]
pub struct SystemState {
    pub temperature: f32,
    pub estoque: String,
    pub porta_aberta: bool,
    pub temperature_threshold: f32,
    pub registered_sensors: HashSet<String>,
    pub registered_actuators: HashSet<String>,
    pub actuator_states: HashMap<String, bool>,
    pub cooler_active: bool,
    pub min_temperature: f32,
    pub cooling_rate: f32,
    pub heating_rate: f32,
}

pub async fn start_server() -> tokio::io::Result<()> {
    let state = Arc::new(Mutex::new(SystemState::default()));
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    println!("🚀 Servidor rodando em 127.0.0.1:8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        let state_clone = Arc::clone(&state);
        
        tokio::spawn(async move {
            println!("📡 Nova conexão de: {:?}", addr);
            handle_connection(socket, state_clone).await;
        });
    }
}

async fn handle_connection(mut socket: TcpStream, state: Arc<Mutex<SystemState>>) {
    let mut buffer = [0; 1024];
    let mut request_data = Vec::new();
    
    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => break,
            Ok(size) => {
                request_data.extend_from_slice(&buffer[..size]);
                
                // Verifica se recebeu o fim da requisição
                if request_data.windows(4).any(|w| w == b"\r\n\r\n") {
                    let request = String::from_utf8_lossy(&request_data);
                    let response = process_request(&request, &state).await;
                    socket.write_all(response.as_bytes()).await.unwrap();
                    request_data.clear();
                }
            }
            Err(_) => break,
        }
    }
    println!("⚠️ Conexão encerrada.");
}

async fn process_request(request: &str, state: &Arc<Mutex<SystemState>>) -> String {
    let lines: Vec<&str> = request.lines().collect();
    
    match lines[0] {
        l if l.starts_with("SENSOR/1.0") => handle_sensor_message(&lines, state).await,
        l if l.starts_with("CLIENT/1.0") => handle_client_message(&lines, state).await,
        l if l.starts_with("ACTUATOR/1.0") => handle_actuator_message(&lines, state).await,
        _ => "\r\n MANAGER/1.0 400 ERRO\r\nContent-Length: 25\r\n\r\nMensagem não reconhecida \r\n".to_string()
    }
}

pub async fn control_loop(state: Arc<Mutex<SystemState>>) {
    let mut rng = rand::thread_rng();
    
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let mut state = state.lock().await;

        // Temperature simulation
        let variation = rng.gen_range(-0.3..0.3);
        let temp_change = if state.cooler_active {
            -state.cooling_rate + variation
        } else {
            state.heating_rate + variation
        };
        
        state.temperature += temp_change;
        
        // Physical limits
        state.temperature = state.temperature.clamp(state.min_temperature, 40.0);
        
        // Continuous temperature logging 🌡️
        println!(
            "🌡️ Current: {:.1}°C | Change: {:.1}°C {}",
            state.temperature,
            temp_change.abs(),
            if temp_change > 0.0 { "🔥" } else { "❄️" }
        );

        // Cooler control with emoji status
        let new_cooler_state = state.temperature > state.temperature_threshold;
        if new_cooler_state != state.cooler_active {
            state.cooler_active = new_cooler_state;
            
            let (emoji, status) = if new_cooler_state {
                ("🚨❄️", "ATIVANDO")
            } else {
                ("🚨💤", "DESATIVANDO")
            };
            
            println!(
                "{} Cooler {} | Threshold: {:.1}°C",
                emoji, status, state.temperature_threshold
            );
            
            // Send command to physical actuator
            send_actuator_command(&state, "C1", new_cooler_state).await;
        }
    }
}