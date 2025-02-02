use crate::server::SystemState;
use std::sync::Arc;
use tokio::sync::Mutex;

// Handler para sensores
pub async fn handle_sensor_data(lines: &[&str], state: &Arc<Mutex<SystemState>>) -> String {
    let mut state = state.lock().await;
    
    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[..] {
            ["TEMPERATURA", value] => state.temperatura_interna = value.parse().unwrap_or(4.0),
            ["PORTA", status] => state.porta_aberta = status.eq_ignore_ascii_case("ABERTA"),
            ["ESTOQUE", percent] => state.nivel_estoque = percent.parse().unwrap_or(0),
            _ => {}
        }
    }
    
    "MANAGER/1.0 200 OK\r\n\r\n".to_string()
}

// Handler para clientes
pub async fn handle_client_command(lines: &[&str], state: &Arc<Mutex<SystemState>>) -> String {
    let mut state = state.lock().await;
    
    match lines.get(1) {
        Some(&"GET_STATUS") => format!(
            "MANAGER/1.0 200 OK\r\n\
            TEMPERATURA: {:.1}\r\n\
            PORTA: {}\r\n\
            REFRIGERADOR: {}\r\n\
            ALARME: {}\r\n\r\n",
            state.temperatura_interna,
            if state.porta_aberta { "ABERTA" } else { "FECHADA" },
            if state.refrigerador_ligado { "LIGADO" } else { "DESLIGADO" },
            if state.alarme_ativado { "ATIVADO" } else { "NORMAL" }
        ),
          Some(line) if line.starts_with("SET_LIMIT") => {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 {
                state.temperatura_ideal = parts[1].parse().unwrap_or(4.0);
                format!("MANAGER/1.0 200 OK\r\nLIMITE: {:.1}\r\n\r\n", state.temperatura_ideal)
                } else {
                    "MANAGER/1.0 400 ERROR\r\n\r\n".to_string()
                }
            }
        _ => "MANAGER/1.0 400 ERROR\r\n\r\n".to_string()
    }
}

pub async fn handle_actuator_command(lines: &[&str], state: &Arc<Mutex<SystemState>>) -> String {
    let mut state = state.lock().await;
    
    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[..] {
            ["REFRIGERADOR", status] => {
                state.refrigerador_ligado = status.eq_ignore_ascii_case("LIGAR");
                return "MANAGER/1.0 200 OK\r\n\r\n".to_string();
            }
            ["LUZ", status] => {
                state.luz_ligada = status.eq_ignore_ascii_case("ACENDER");
                "MANAGER/1.0 200 OK\r\n\r\n".to_string()
            }
            _ => "MANAGER/1.0 400 ERROR\r\n\r\n".to_string()
        };
    }
    
    "MANAGER/1.0 200 OK\r\n\r\n".to_string()
}