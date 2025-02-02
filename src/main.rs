use std::sync::Arc;
use tokio::sync::Mutex;
mod handlers;
mod server;
use crate::server::SystemState; // Adicione esta linha
use crate::server::control_loop;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(SystemState::new())); // Usar new() ao invés de default()
    
    // Inicia loop de controle em segundo plano
    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        control_loop(state_clone).await;
    });

    if let Err(e) = server::start_server(state).await {
        eprintln!("Erro ao iniciar o servidor: {}", e);
    }
}